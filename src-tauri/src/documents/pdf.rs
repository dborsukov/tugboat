use std::path::PathBuf;

use rust_decimal::Decimal;
use typst::{
    diag::{FileError, FileResult, Warned},
    foundations::{Bytes, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, LibraryExt, World,
};
use typst_layout::PagedDocument;

use super::model::{DocClient, DocTable, DocumentData, DocumentKind};
use crate::{
    database::clients::structs::ClientType,
    error::{AppError, AppResult},
};

// Reminders

// 1. Every user value goes through `esc`. Never interpolate raw text into markup:
//    use `esc`, `esc_wide`, `esc_limited(s, limit)`, `esc_line(s, limit)`, `esc_narrow`.
//    They escape `\ # [ ] * _ $ \``, fold newlines, and apply the long-token policy.
// 2. Amounts only via `money()` — it inserts non-breaking spaces (`1 023,45` must never wrap).
// 3. Measure unbounded. `measure(body).width`, never `measure(body, width: X)`: with an
//    explicit `width` Typst returns the already wrapped width, so overflow becomes undetectable.
// 4. Stay inline: `layout(...)` and `scale(...)` are block-level: inside a line they split
//    it and add block spacing. Inline-safe shrinking is `context` + `measure` + `text(size:)`.
// 5. Use `auto` columns to measure, `1fr` columns to render. `auto` gives natural content width
//    (`measure`); `1fr` spans the container so labels/amounts stay aligned to the box edges.

// ---------------------------------------------------------------------------
// Design tokens — mirror of the Typst reference templates.
// ---------------------------------------------------------------------------

const CLR_ACCENT: &str = "rgb(\"#17427f\")";
const CLR_ACC_SOFT: &str = "rgb(\"#EDF1F5\")";
const CLR_MUTED: &str = "rgb(\"#5B6772\")";
const CLR_ZEBRA: &str = "rgb(\"#F7F8FA\")";
const CLR_BG_RULES: &str = "rgb(\"#FAFAFB\")";

const STROKE_THIN: &str = "0.5pt + rgb(\"#D7DCE2\")";
const STROKE_REG: &str = "0.6pt + rgb(\"#D7DCE2\")";

// ---------------------------------------------------------------------------
// Embedded fonts: IBM Plex Sans + IBM Plex Mono (SIL OFL 1.1 — fonts/OFL.txt)
// ---------------------------------------------------------------------------

const FONT_IBM_PLEX_SANS: &[u8] = include_bytes!("../../fonts/IBMPlexSans-Variable.ttf");
const FONT_IBM_PLEX_SANS_ITALIC: &[u8] =
    include_bytes!("../../fonts/IBMPlexSans-Italic-Variable.ttf");
const FONT_IBM_PLEX_MONO_REGULAR: &[u8] = include_bytes!("../../fonts/IBMPlexMono-Regular.ttf");
const FONT_IBM_PLEX_MONO_ITALIC: &[u8] = include_bytes!("../../fonts/IBMPlexMono-Italic.ttf");
const FONT_IBM_PLEX_MONO_BOLD: &[u8] = include_bytes!("../../fonts/IBMPlexMono-Bold.ttf");
const FONT_IBM_PLEX_MONO_BOLD_ITALIC: &[u8] =
    include_bytes!("../../fonts/IBMPlexMono-BoldItalic.ttf");

const EMBEDDED_FONTS: [&[u8]; 6] = [
    FONT_IBM_PLEX_SANS,
    FONT_IBM_PLEX_SANS_ITALIC,
    FONT_IBM_PLEX_MONO_REGULAR,
    FONT_IBM_PLEX_MONO_ITALIC,
    FONT_IBM_PLEX_MONO_BOLD,
    FONT_IBM_PLEX_MONO_BOLD_ITALIC,
];

// ---------------------------------------------------------------------------
// Typst specific formatting helpers
// ---------------------------------------------------------------------------

/// Minimum length of a token that is wrapped in `#shrink(...)`.
///
/// Shorter tokens stay plain text, so that hyphenation (`hyphenate`) works for
/// words instead of shrinking them.
const FIT_MIN_CHARS: usize = 20;

/// Minimum length of a token that is wrapped in `#shrink(...)` in the columns of
/// the details block.
///
/// The columns there are narrow (120–180pt in a 3–4 column block), and an
/// already 12-character token takes up more than half the width — shrinking it
/// is safer than hoping that it fits.
const FIT_MIN_CHARS_NARROW: usize = 12;

/// Length after which a token is considered "junk" and wrapped, not shrunk:
/// real IBANs (up to 34 characters), VINs, codes and numbers are shorter.
///
/// Shrinking a 200-character token would mean a font size of fractions of a
/// point — such text looks like a grey stripe. Such tokens are cut into chunks
/// with a zero-width space between them: they wrap across lines and stay
/// readable.
const BREAK_MIN_CHARS: usize = 36;

/// Length of a chunk that an over-long token is cut into. 10 characters fit
/// even into the narrowest table column.
const BREAK_CHUNK_CHARS: usize = 10;

/// Name of the typst variable that holds the actual column width (see `layout`).
const AVAILABLE_WIDTH: &str = "avail";

/// Token width limit in narrow columns (in points).
///
/// The narrow spots of the document: values in the details grid (about 150pt),
/// the client blocks in the act and the labels. Real IBANs (up to 29
/// characters), VINs and codes are narrower than this limit and are not shrunk,
/// while everything longer is shrunk — otherwise it goes beyond the page.
const FIT_LIMIT_BODY: &str = "150pt";

/// Token width limit in wide blocks: the business header, the title, the defects.
///
/// The columns there are wider (over 250pt), so even a long IBAN fits entirely
/// and needs no shrinking.
const FIT_LIMIT_WIDE: &str = "220pt";

/// Width limit for the content of table columns (in points).
///
/// Table cells are measured at unbounded width, so the limit is set explicitly
/// — otherwise a long token widens the `auto`/`1fr` column and the table runs
/// off the page.
const FIT_LIMIT_CODE: &str = "80pt";
const FIT_LIMIT_UNIT: &str = "70pt";
const FIT_LIMIT_NAME: &str = "200pt";

/// Escape user-provided text for inclusion in Typst markup.
///
/// Long unbreakable tokens (IBANs, VINs, codes, links) are additionally wrapped
/// in `#shrink(...)`: Typst scales them down to the given limit instead of
/// pushing the text beyond the page (typst has no global "do not overflow"
/// setting and does not warn about overflow).
fn esc(s: &str) -> String {
    esc_with_limit(s, FIT_LIMIT_BODY)
}

/// Like [`esc`], but long tokens are shrunk to the explicit limit `limit`.
fn esc_limited(s: &str, limit: &str) -> String {
    esc_with_limit(s, limit)
}

/// Like [`esc`], but for wide blocks (the business header, the title, defects).
fn esc_wide(s: &str) -> String {
    esc_with_limit(s, FIT_LIMIT_WIDE)
}

/// Like [`esc`], but for the columns of the details block: the shrink limit is
/// the actual column width (`avail`), and tokens are shrunk starting from
/// [`FIT_MIN_CHARS_NARROW`] characters.
///
/// In a column 120–180pt wide (a details block with 3–4 columns) a VIN is
/// 17 characters and 98pt, so the usual limit [`FIT_MIN_CHARS`] would leave it
/// unshrunk and it would run outside the column.
fn esc_narrow(s: &str) -> String {
    esc_with_policy(s, AVAILABLE_WIDTH, FIT_MIN_CHARS_NARROW)
}

/// Escapes a whole line and shrinks it as a single unit — so that a label and a
/// value ("Р/Р: UA…", "Банк (МФО): …") stay on one line, shrinking together if
/// needed.
fn esc_line(s: &str, limit: &str) -> String {
    format!("#shrink({limit})[{}]", escape_markup(s))
}

fn esc_with_limit(s: &str, limit: &str) -> String {
    esc_with_policy(s, limit, FIT_MIN_CHARS)
}

fn esc_with_policy(s: &str, limit: &str, fit_min_chars: usize) -> String {
    escape_markup(s)
        .split(' ')
        .map(|token| {
            let len = token.chars().count();
            if len >= BREAK_MIN_CHARS {
                break_token(token, limit)
            } else if len >= fit_min_chars {
                format!("#shrink({limit})[{token}]")
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Escapes text for insertion into typst markup.
fn escape_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '#' => out.push_str("\\#"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '*' => out.push_str("\\*"),
            '_' => out.push_str("\\_"),
            '$' => out.push_str("\\$"),
            '`' => out.push_str("\\`"),
            // Literal newlines are not allowed inside table cells.
            '\n' | '\r' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// Cuts an over-long token into chunks separated by a zero-width space
/// (`U+200B` — a line break opportunity for typst) and wraps them in a box of
/// width `limit`.
///
/// `box` is essential here: break opportunities alone do not limit the maximum
/// content width, so an `auto` column stretches to the full length of the token
/// (measured: 487pt versus 90pt) and the table breaks. The box sets the width
/// inside which the text wraps across lines at full font size.
fn break_token(token: &str, limit: &str) -> String {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for c in token.chars() {
        current.push(c);
        if current.chars().count() == BREAK_CHUNK_CHARS {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }

    let body = chunks
        .iter()
        .map(|chunk| format!("#shrink({limit})[{chunk}]"))
        .collect::<Vec<_>>()
        .join("\u{200B}");
    format!("#box(width: {limit})[{body}]")
}

/// Typst non-wrapping space
fn money(v: Decimal) -> String {
    super::model::format_money(v).replace(' ', "\u{00A0}")
}

/// Blank space meant to be filled out by hand
fn blank_line(width: &str) -> String {
    format!("#box(width: {width}, line(length: 100%, stroke: {STROKE_REG}))")
}

// ---------------------------------------------------------------------------
// Header (business details)
// ---------------------------------------------------------------------------

/// Two-column header: the business name, physical address and phone on the
/// left, the full legal details on the right.
fn business_header_markup(data: &DocumentData) -> String {
    let mut out = String::new();

    let Some(info) = &data.company else {
        out.push_str(&format!(
            "#text(fill: {CLR_MUTED})[{}]\n\n",
            esc_wide("Дані бізнесу не заповнено")
        ));
        return out;
    };

    let business_name = {
        let name = super::model::company_name_line(info);
        let name = name.trim().to_string();
        if name.is_empty() {
            "Станція технічного обслуговування (СТО)".to_string()
        } else {
            name
        }
    };
    let full_legal = super::model::company_full_legal_line(info);

    // Left column: business name, physical address, phone.
    let mut left_lines = vec![format!(
        "#text(weight: \"bold\", size: 10.5pt)[{}]",
        esc_wide(&business_name)
    )];
    let mut contact_lines = Vec::new();
    if let Some(addr) = info
        .physical_address
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        contact_lines.push(format!(
            "#text(weight: \"bold\", [Адреса: ]) {}",
            esc_wide(addr)
        ));
    }
    if let Some(phone) = info
        .phone
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        contact_lines.push(format!(
            "#text(weight: \"bold\", [Телефон: ]) {}",
            esc_wide(phone)
        ));
    }
    if !contact_lines.is_empty() {
        left_lines.push("#v(0.4em)".to_string());
        left_lines.push(contact_lines.join(" \\\n"));
    }

    // Right column: full legal name, code, legal address, account.
    let mut right_lines = Vec::new();
    if !full_legal.trim().is_empty() {
        right_lines.push(format!(
            "#text(weight: \"bold\", size: 10.5pt)[{}]",
            esc_wide(&full_legal)
        ));
    }
    let mut details = Vec::new();
    if let Some((label, value)) = super::model::company_code_parts(info) {
        details.push(esc_wide(&format!("{label} {value}")));
    }
    if let Some(addr) = info
        .legal_address
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        details.push(esc_wide(addr));
    }
    for bank_line in super::model::company_bank_lines(info) {
        details.push(esc_line(&bank_line, AVAILABLE_WIDTH));
    }
    if !details.is_empty() {
        right_lines.push("#v(0.4em)".to_string());
        right_lines.push(details.join(" \\\n"));
    }

    out.push_str("#grid(\n  columns: (1fr, 1fr),\n  gutter: 0pt,\n");
    out.push_str(&format!("  grid.vline(x: 1, stroke: {STROKE_REG}),\n"));
    out.push_str(&format!("  block[{}],\n", left_lines.join("\n")));
    // The width of the details column is known only in place — `layout` hands
    // it over as `avail` for the bank lines.
    out.push_str(&format!(
        "  block(inset: (left: 1.4em))[#layout(sz => {{ let {AVAILABLE_WIDTH} = sz.width\n[{}] }})],\n",
        right_lines.join("\n")
    ));
    out.push_str(")\n");
    out
}

// ---------------------------------------------------------------------------
// Document title
// ---------------------------------------------------------------------------

fn title_block_markup(data: &DocumentData) -> String {
    let mut out = String::new();
    out.push_str("#block(\n  width: 100%,\n");
    out.push_str(&format!(
        "  fill: {CLR_ACC_SOFT},\n  inset: (x: 1em, y: 0.6em),\n  radius: 2pt,\n  breakable: false,\n)[\n"
    ));
    out.push_str("  #align(center)[\n");
    out.push_str(&format!(
        "    #text(size: 13.5pt, weight: \"bold\", fill: {CLR_ACCENT})[{}]\n",
        esc_wide(&data.title)
    ));
    out.push_str("    #linebreak()\n");

    match data.kind {
        DocumentKind::NaryadZamovlennya => match &data.doc_date {
            Some(date) => out.push_str(&format!(
                "    #text(size: 10.5pt, fill: {CLR_MUTED})[від {}]\n",
                esc_wide(date)
            )),
            None => out.push_str(&format!(
                "    #text(size: 10.5pt, fill: {CLR_MUTED})[від ]{} \n",
                blank_line("15%")
            )),
        },
        DocumentKind::AktVykonanykhRobit => {
            match &data.doc_date {
                Some(date) => out.push_str(&format!(
                    "    #text(size: 10.5pt, fill: {CLR_MUTED})[від {}, підстава: наряд-замовлення № {}]\n",
                    esc_wide(date),
                    esc_wide(&data.work_order_id)
                )),
                None => out.push_str(&format!(
                    "    #text(size: 10.5pt, fill: {CLR_MUTED})[від ]{}\n    #text(size: 10.5pt, fill: {CLR_MUTED})[, підстава: наряд-замовлення № {}]\n",
                    blank_line("15%"),
                    esc_wide(&data.work_order_id)
                )),
            }
            out.push_str("    #linebreak()\n");
            match &data.doc_location {
                Some(location) => out.push_str(&format!(
                    "    #text(size: 10.5pt, fill: {CLR_MUTED})[місце складання: {}]\n",
                    esc_wide(location)
                )),
                None => out.push_str(&format!(
                    "    #text(size: 10.5pt, fill: {CLR_MUTED})[місце складання: ]{} \n",
                    blank_line("55%")
                )),
            }
        }
    }
    out.push_str("  ]\n]\n");
    out
}

// ---------------------------------------------------------------------------
// Car and clients
// ---------------------------------------------------------------------------

/// Full client details: name, an individual's phone, code, legal address,
/// director and bank — each on a separate line.
fn client_detail_lines(client: &DocClient) -> Vec<String> {
    let mut lines = vec![format!(
        "#text(weight: \"bold\")[{}]",
        esc_narrow(&super::model::legal_display_name(client))
    )];
    // An individual has no details — their contact is the phone.
    if client.client_type == ClientType::Individual && !client.phone.is_empty() {
        lines.push(esc_narrow(&client.phone));
    }
    if let Some(code) = super::model::client_tax_code_line(client) {
        lines.push(esc_narrow(&code));
    }
    if let Some(address) = client
        .legal_address
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        lines.push(esc_narrow(address));
    }
    if let Some(director) = client
        .director_pib
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        lines.push(format!("Керівник:{}", esc_narrow(director)));
    }
    for bank_line in super::model::client_bank_info_lines(client) {
        lines.push(esc_line(&bank_line, AVAILABLE_WIDTH));
    }
    lines
}

fn labeled_rows_markup(rows: &[(String, String)]) -> String {
    let mut out = String::from(
        "    #grid(\n      columns: (auto, 1fr),\n      column-gutter: 0.8em,\n      row-gutter: 0.5em,\n",
    );
    for (label, value) in rows {
        out.push_str(&format!(
            "      text(weight: \"bold\")[{label}], block()[#layout(sz => {{ let {AVAILABLE_WIDTH} = sz.width\n[{value}] }})],\n"
        ));
    }
    out.push_str("    )\n");
    out
}

/// A column heading line and the hairline rule under it.
fn column_heading_markup(header: &str) -> String {
    format!(
        "    #text(weight: \"bold\", size: 10pt, fill: {CLR_ACCENT})[{header}]\n    #v(0.3em)\n    #line(length: 100%, stroke: {STROKE_THIN})\n    #v(0.5em)\n"
    )
}

/// The largest number of fields in one row of the horizontal grid, when a
/// section takes the full page width.
const MAX_FIELDS_PER_ROW: usize = 4;

/// Font size of the labels in the horizontal field grid: smaller than the values
/// (9.5pt), but large enough for a label to read as a line of its own.
const FIELD_LABEL_SIZE: &str = "8.5pt";

/// Section fields laid out as a horizontal grid: a label above a value, several
/// fields per row — this way the strip of fields uses the page width instead of
/// stretching vertically as in a narrow column.
fn horizontal_fields_markup(rows: &[(String, String)]) -> String {
    let columns = rows.len().clamp(1, MAX_FIELDS_PER_ROW);
    let mut out = format!(
        "    #grid(\n      columns: ({}),\n      column-gutter: 1.2em,\n      row-gutter: 0.8em,\n",
        vec!["1fr"; columns].join(", ")
    );
    for (label, value) in rows {
        // The label stands above the value, so the trailing colon is redundant.
        out.push_str(&format!(
            "      grid(\n        columns: 1fr,\n        row-gutter: 0.5em,\n        text(weight: \"bold\", size: {FIELD_LABEL_SIZE})[{}],\n        block()[#layout(sz => {{ let {AVAILABLE_WIDTH} = sz.width\n[{value}] }})],\n      ),\n",
            label.trim_end_matches(':')
        ));
    }
    out.push_str("    )\n");
    out
}

/// The car section: a heading and the details of the car.
///
/// `standalone` — the section stands on its own across the full page width
/// (when there are several parties in the details block), and the fields are
/// laid out horizontally. Otherwise the section stands as a column next to the
/// owner, where there is no room for a horizontal grid.
fn vehicle_block_markup(data: &DocumentData, standalone: bool) -> String {
    let car_rows = vec![
        (
            "Марка/модель:".to_string(),
            esc_narrow(&data.car_make_model),
        ),
        (
            "Рік випуску:".to_string(),
            esc_narrow(&data.car_year.to_string()),
        ),
        (
            "Пробіг:".to_string(),
            format!("{} км.", esc_narrow(&data.odometer_in.to_string())),
        ),
        (
            "Державний номер:".to_string(),
            esc_narrow(&data.car_license_plate),
        ),
        ("VIN-код:".to_string(), esc_narrow(&data.car_vin)),
    ];

    let mut out = String::new();
    out.push_str(&column_block_open("", standalone, standalone));
    out.push_str(&column_heading_markup("Транспортний засіб"));
    out.push_str(&if standalone {
        horizontal_fields_markup(&car_rows)
    } else {
        labeled_rows_markup(&car_rows)
    });
    out.push_str(column_block_close(standalone));
    out
}

/// The representative section: the driver's full name and phone, below the car
/// and client sections — whatever the layout of the details block.
///
/// A driver is only specified for sole proprietors and organisations (see the
/// order form); if none is given, the section is absent entirely.
fn driver_block_markup(data: &DocumentData) -> Option<String> {
    let mut rows = Vec::new();
    if let Some(name) = data.driver_name() {
        rows.push(("ПІБ:".to_string(), esc_narrow(&name)));
    }
    if let Some(phone) = data
        .driver_phone
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        rows.push(("Телефон:".to_string(), esc_narrow(phone)));
    }
    if rows.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str(&column_block_open("", true, true));
    out.push_str(&column_heading_markup("Представник / водій"));
    out.push_str(&horizontal_fields_markup(&rows));
    out.push_str(column_block_close(true));
    Some(out)
}

/// A client column: a line with the name of the party ("Власник", "Замовник"…)
/// and the full details under it.
///
/// `first` — the first column of the grid: there is no line to its left, so an
/// inset would only shift the whole client block to the right of the page edge.
fn client_column_markup(header: &str, client: &DocClient, first: bool, last: bool) -> String {
    let lines = client_detail_lines(client).join(" \\\n");
    let left_inset = if first { "" } else { "1.4em" };
    let mut out = String::new();
    out.push_str(&column_block_open(left_inset, last, false));
    out.push_str(&column_heading_markup(header));
    out.push_str(&format!(
        "    #layout(sz => {{ let {AVAILABLE_WIDTH} = sz.width\n[{lines}] }})\n"
    ));
    out.push_str(column_block_close(false));
    out
}

/// Opening of a details block column: a left inset (client columns are separated
/// by a vertical line) and a right one — so that the text does not touch the
/// line.
///
/// `top_level` — the column stands directly in the markup (the car section above
/// the grid) rather than among the arguments of `#grid(...)`: in markup the
/// directive must be written with `#`, in argument code without it.
fn column_block_open(left_inset: &str, last: bool, top_level: bool) -> String {
    let mut inset = String::new();
    if !left_inset.is_empty() {
        inset.push_str(&format!("left: {left_inset}"));
    }
    if !last {
        if !inset.is_empty() {
            inset.push_str(", ");
        }
        inset.push_str("right: 0.9em");
    }
    let directive = if top_level { "#block" } else { "block" };
    if inset.is_empty() {
        format!("  {directive}[\n")
    } else {
        format!("  {directive}(inset: ({inset}))[\n")
    }
}

/// Closing a column: inside a grid a comma is required after `]` (the cell
/// separator), while in markup it would become document text.
fn column_block_close(top_level: bool) -> &'static str {
    if top_level {
        "  ]\n"
    } else {
        "  ],\n"
    }
}

/// The details block: the car and the clients.
///
/// As long as all the parties (owner, customer, payer) are the same person, the
/// details stand in two columns: the car and the owner. As soon as a second
/// party appears, the car is moved out into a separate full-width section, and
/// the client columns share the page width between them.
fn info_grid_markup(data: &DocumentData) -> String {
    let owner = &data.owner;
    // The payer is only rendered in the acceptance act.
    let payer_supported = data.kind == DocumentKind::AktVykonanykhRobit;

    let customer = data.customer.as_ref().filter(|c| c.id != owner.id);
    let payer = data
        .payer
        .as_ref()
        .filter(|p| payer_supported && p.id != owner.id);

    // Customer and payer are the same person: one column with a double heading.
    let (customer_column, payer_column, customer_is_payer) = match (customer, payer) {
        (Some(customer), Some(payer)) if customer.id == payer.id => (Some(customer), None, true),
        (customer, payer) => (customer, payer, false),
    };

    // The owner heading covers all the parties that the owner itself is.
    let owner_header = {
        let mut header = String::from("Власник");
        if data.customer.as_ref().is_none_or(|c| c.id == owner.id) {
            header.push_str(" / Замовник");
        }
        if payer_supported && data.payer.as_ref().is_none_or(|p| p.id == owner.id) {
            header.push_str(" / Платник");
        }
        header
    };

    // Client columns from left to right: owner, customer, payer.
    let mut clients: Vec<(String, &DocClient)> = vec![(owner_header, owner)];
    if let Some(customer) = customer_column {
        let header = if customer_is_payer {
            "Замовник / Платник"
        } else {
            "Замовник"
        };
        clients.push((header.to_string(), customer));
    }
    if let Some(payer) = payer_column {
        clients.push(("Платник".to_string(), payer));
    }

    // The representative section stands under the car and client sections
    // whatever the layout; if no driver is given, it is absent.
    let driver = driver_block_markup(data);

    // One party: the car and the owner side by side.
    if let [(header, client)] = clients.as_slice() {
        let mut out = String::new();
        out.push_str("#grid(\n  columns: (1fr, 1fr),\n  gutter: 0pt,\n");
        out.push_str(&format!("  grid.vline(x: 1, stroke: {STROKE_REG}),\n"));
        out.push_str(&vehicle_block_markup(data, false));
        out.push_str(&client_column_markup(header, client, false, true));
        out.push_str(")\n");
        if let Some(driver) = &driver {
            out.push_str("#v(0.8em)\n");
            out.push_str(driver);
        }
        return out;
    }

    // Several parties: the car as a full-width section, the clients under it.
    let last = clients.len() - 1;
    let mut out = vehicle_block_markup(data, true);
    out.push_str("#v(0.8em)\n");
    out.push_str("#grid(\n  columns: (");
    out.push_str(&vec!["1fr"; clients.len()].join(", "));
    out.push_str("),\n  gutter: 0pt,\n");
    for x in 1..clients.len() {
        out.push_str(&format!("  grid.vline(x: {x}, stroke: {STROKE_REG}),\n"));
    }
    for (index, (header, client)) in clients.iter().enumerate() {
        out.push_str(&client_column_markup(
            header,
            client,
            index == 0,
            index == last,
        ));
    }
    out.push_str(")\n");
    if let Some(driver) = &driver {
        out.push_str("#v(0.8em)\n");
        out.push_str(driver);
    }
    out
}

// ---------------------------------------------------------------------------
// Services and parts table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableSection {
    Labor,
    Parts,
}

const NARYAD_HEADERS: [&str; 9] = [
    "№",
    "Код",
    "Назва",
    "Кількість",
    "Од.\\ вим.",
    "Ціна,\\ грн.",
    "Знижка",
    "Відпускна\\ ціна, грн.",
    "Сума,\\ грн.",
];

const AKT_HEADERS: [&str; 9] = [
    "№",
    "Код",
    "Назва",
    "Кіль-\\ кість",
    "Од.\\ вим.",
    "Ціна,\\ грн.",
    "Знижка",
    "Відпускна\\ ціна, грн.",
    "Сума,\\ грн.",
];

/// The section heading and the label of the total row, depending on the
/// document type.
fn table_labels(kind: DocumentKind, section: TableSection) -> (&'static str, &'static str) {
    match (kind, section) {
        (_, TableSection::Labor) => ("Послуги / роботи", "Всього"),
        (_, TableSection::Parts) => ("Запчастини та витратні матеріали", "Всього"),
    }
}

/// The rows of one table section: the section heading, the items and the total.
fn table_section_rows(kind: DocumentKind, table: &DocTable, section: TableSection) -> String {
    let (section_title, total_label) = table_labels(kind, section);
    let mut out = String::new();

    // Section separator.
    out.push_str(&format!(
        "  table.cell(colspan: 9, fill: {CLR_ACC_SOFT}, inset: (x: 8pt, y: 6pt))[#text(weight: \"bold\", size: 9.5pt, fill: {CLR_ACCENT})[{}]],\n",
        esc(section_title)
    ));

    if table.rows.is_empty() {
        out.push_str(&format!(
            "  table.cell(colspan: 9, align: center, fill: white)[{}],\n",
            esc("Немає записів")
        ));
    }
    for (i, row) in table.rows.iter().enumerate() {
        let fill = if i % 2 == 0 { "white" } else { CLR_ZEBRA };
        let cells = [
            format!("[{}]", esc(&(i + 1).to_string())),
            format!("[{}]", esc_limited(&row.code, FIT_LIMIT_CODE)),
            format!("[{}]", esc_limited(&row.name, FIT_LIMIT_NAME)),
            format!("[{}]", esc(&super::model::format_qty(row.quantity))),
            format!("[{}]", esc_limited(&row.unit, FIT_LIMIT_UNIT)),
            format!("[{}]", money(row.price)),
            format!("[{}]", esc(&row.discount_text)),
            format!("[{}]", money(row.discounted_price)),
            format!("[*{}*]", money(row.sum)),
        ];
        for cell in cells {
            out.push_str(&format!("  table.cell(fill: {fill}){cell},\n"));
        }
    }

    // Section total.
    out.push_str(&format!(
        "  table.cell(colspan: 8, align: right, inset: (x: 5pt, y: 6pt), stroke: (top: {STROKE_REG}))[#text(weight: \"bold\", size: 9pt, fill: {CLR_ACCENT})[{}]],\n",
        esc(total_label)
    ));
    out.push_str(&format!(
        "  table.cell(align: right, inset: (x: 5pt, y: 6pt), stroke: (top: {STROKE_REG}))[#text(weight: \"bold\")[{}]],\n",
        money(table.total)
    ));

    out
}

/// The combined services and parts table: one header, shared columns and a total
/// for each section.
fn items_table_markup(kind: DocumentKind, labor: &DocTable, parts: &DocTable) -> String {
    let headers: &[&str; 9] = if kind == DocumentKind::AktVykonanykhRobit {
        &AKT_HEADERS
    } else {
        &NARYAD_HEADERS
    };

    let mut out = String::new();
    // `fitl` in the cells stops the `auto`/`1fr` columns from pushing the table
    // beyond the page; `clip` is extra protection against clipping what has
    // already been drawn.
    out.push_str("#block(\n  radius: 2pt,\n  clip: true,\n)[#table(\n");
    out.push_str("  columns: (auto, auto, 1fr, auto, auto, auto, auto, auto, auto),\n");
    out.push_str(&format!("  stroke: {STROKE_THIN},\n"));
    out.push_str("  inset: (x: 5pt, y: 4.5pt),\n");
    out.push_str("  align: (center, center, left, center, center, right, right, right, right),\n");

    // A shared header for both sections.
    out.push_str("  table.header(");
    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "table.cell(fill: {CLR_ACCENT}, inset: (x: 4pt, y: 6pt), align: center)[#text(fill: white, weight: \"bold\")[{}]]",
            header
        ));
    }
    out.push_str("),\n");

    // Section 1: services (labour), section 2: parts.
    out.push_str(&table_section_rows(kind, labor, TableSection::Labor));
    out.push_str(&table_section_rows(kind, parts, TableSection::Parts));

    out.push_str(")]\n");
    out
}

// ---------------------------------------------------------------------------
// Totals and the amount in words
// ---------------------------------------------------------------------------

fn totals_markup(data: &DocumentData) -> String {
    let t = &data.totals;
    let mut rows: Vec<(String, String)> = vec![(
        "Загальна сума:".to_string(),
        format!("{} грн.", money(t.subtotal)),
    )];
    if t.order_discount > Decimal::ZERO {
        // For a percentage discount we show the percentage itself — as for VAT.
        let discount_label = if t.order_discount_is_percentage {
            format!(
                "Знижка за замовленням ({}%):",
                super::model::format_qty(t.order_discount_percent)
            )
        } else {
            "Знижка за замовленням:".to_string()
        };
        rows.push((discount_label, format!("-{} грн.", money(t.order_discount))));
        rows.push((
            "Загальна сума зі знижкою:".to_string(),
            format!("{} грн.", money(t.grand_total)),
        ));
    }

    if data.vat_payer && t.prices_include_vat {
        rows.push((
            "Без ПДВ:".to_string(),
            format!("{} грн.", money(t.grand_total - t.vat_amount)),
        ));
        rows.push((
            format!("ПДВ ({}%):", super::model::format_qty(t.vat_rate)),
            format!("{} грн.", money(t.vat_amount)),
        ));
    } else if data.vat_payer {
        rows.push((
            format!("Сума ПДВ ({}%):", super::model::format_qty(t.vat_rate)),
            format!("{} грн.", money(t.vat_amount)),
        ));
    } else {
        rows.push(("Сума ПДВ:".to_string(), "Без ПДВ".to_string()));
    }

    let mut out = String::new();
    out.push_str("#v(0.8em)\n#align(right)[\n");
    // The totals box grows to fit the content: the width is set by `measure`
    // over the rows, so no label wraps onto a second line. The limit is the page
    // width: if the content is wider, the box does not go beyond the margins.
    out.push_str("#layout(size => {\n");
    // Grids for measuring — `auto` columns, i.e. the natural content width.
    out.push_str("  let measured = grid(\n    columns: (auto, auto),\n    column-gutter: 1.5em,\n    row-gutter: 0.75em,\n    align: (left, right),\n");
    out.push_str(&totals_row_cells(&rows));
    out.push_str("  )\n");
    out.push_str("  let measured_total = grid(\n    columns: (auto, auto),\n    column-gutter: 1.5em,\n    align: (left, right),\n");
    out.push_str(&totals_total_cells(t.total_with_vat));
    out.push_str("  )\n");
    out.push_str("  let width = calc.min(\n    calc.max(measure(measured).width, measure(measured_total).width) + 2 * text.size,\n    size.width,\n  )\n");
    out.push_str(&format!(
        "  block(\n    width: width,\n    stroke: {STROKE_REG},\n    radius: 2pt,\n    inset: 0pt,\n    breakable: false,\n  )[\n"
    ));
    // Render grids across the full width of the box: the labels are pushed to
    // the left edge, the amounts to the right, no matter which row is the
    // longest.
    out.push_str("    #block(inset: (x: 1em, top: 0.8em, bottom: 0.2em))[\n      #grid(\n        columns: (1fr, auto),\n        column-gutter: 1.5em,\n        row-gutter: 0.75em,\n        align: (left, right),\n");
    out.push_str(&totals_row_cells(&rows));
    out.push_str("      )\n    ]\n");
    out.push_str(&format!(
        "    #block(fill: {CLR_ACC_SOFT}, inset: (x: 1em, y: 0.85em), radius: (bottom: 2pt))[\n      #grid(\n        columns: (1fr, auto),\n        column-gutter: 1.5em,\n        align: (left, right),\n"
    ));
    out.push_str(&totals_total_cells(t.total_with_vat));
    out.push_str("      )\n    ]\n  ]\n})\n]\n");
    out
}

/// The cells of the total rows (label + amount).
///
/// Used twice: in the grid of `auto` columns to measure the natural width, and
/// in the full-width grid of the box for rendering.
fn totals_row_cells(rows: &[(String, String)]) -> String {
    let mut out = String::new();
    for (label, value) in rows {
        out.push_str(&format!(
            "    text(fill: {CLR_MUTED})[{}], text()[{}],\n",
            esc(label),
            esc(value)
        ));
    }
    out
}

/// The cells of the "Всього до сплати" strip.
fn totals_total_cells(total_with_vat: Decimal) -> String {
    format!(
        "    box[#text(weight: \"bold\", fill: {CLR_ACCENT})[Всього до сплати:]],\n    text(weight: \"bold\", size: 11pt, fill: {CLR_ACCENT})[{} грн.],\n",
        money(total_with_vat)
    )
}

// ---------------------------------------------------------------------------
// Work order terms
// ---------------------------------------------------------------------------

fn terms_block_markup(data: &DocumentData) -> Option<String> {
    let mut rows = Vec::new();
    if let Some(received_at) = data.received_at.as_deref() {
        rows.push(("Прийнято:".to_string(), esc_wide(received_at)));
    }
    if let Some(planned) = data.planned_completion_at.as_deref() {
        rows.push((
            "Орієнтовний термін виконання:".to_string(),
            esc_wide(planned),
        ));
    }
    if rows.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str(&column_block_open("", true, true));
    out.push_str(&column_heading_markup("Терміни"));
    out.push_str(&horizontal_fields_markup(&rows));
    out.push_str(column_block_close(true));
    Some(out)
}

const OLD_PARTS_CLAUSE: &str =
    "Зняті/старі запчастини необхідно(підкресліть одне): повернути замовнику / залишити виконавцю";

fn old_parts_clause_markup() -> String {
    format!(
        "#text(weight: \"regular\")[{}]\n",
        esc_wide(OLD_PARTS_CLAUSE)
    )
}

// ---------------------------------------------------------------------------
// Defects found on acceptance and recommendations
// ---------------------------------------------------------------------------

/// A block with a heading and a ruled field for text.
///
/// Shared by the "defects found on acceptance" in the work order and the
/// "general recommendations" in the act: the same box, the same lines for
/// handwritten text.
fn ruled_text_block_markup(header: &str, text: &str) -> String {
    let mut out = String::new();
    out.push_str("#block(breakable: false)[\n");
    out.push_str(&format!(
        "#text(weight: \"regular\")[{}]\n",
        esc_wide(header)
    ));
    out.push_str("#v(0.6em)\n");
    out.push_str("#layout(size => {\n");
    out.push_str("  let line-height = 9.5pt * 2\n");
    out.push_str("  let font-size = 9.5pt\n");
    out.push_str("  let inset-x = 1em\n");
    out.push_str(&format!("  let content = [ {} ]\n", esc_wide(text)));
    out.push_str("  let formatted-content = [\n");
    out.push_str("    #set text(\n      size: font-size,\n      style: \"italic\",\n      top-edge: 0.8 * font-size,\n      bottom-edge: -0.2 * font-size,\n    )\n");
    out.push_str("    #set par(leading: line-height - font-size)\n");
    out.push_str("    #content\n");
    out.push_str("  ]\n");
    out.push_str("  let avail-width = size.width - (2 * inset-x)\n");
    out.push_str("  let text-metrics = measure(block(width: avail-width, formatted-content))\n");
    out.push_str("  let lines-count = calc.max(2, calc.ceil(text-metrics.height / line-height))\n");
    out.push_str("  block(\n    width: 100%,\n    height: (lines-count + 1) * line-height,\n");
    out.push_str(&format!("    stroke: {STROKE_REG},\n"));
    out.push_str("    radius: 2pt,\n");
    out.push_str("    inset: (x: inset-x, top: 0pt, bottom: 0pt),\n");
    out.push_str("    [\n");
    out.push_str("      #place(top + left)[\n");
    out.push_str("        #for i in range(1, lines-count + 1) {\n");
    out.push_str(&format!(
        "          place(top + left, dy: i * line-height, line(length: 100%, stroke: {STROKE_THIN}))\n"
    ));
    out.push_str("        }\n");
    out.push_str("      ]\n");
    out.push_str("  #v(line-height - (1.2 * font-size))\n");
    out.push_str("      #formatted-content\n");
    out.push_str("    ],\n");
    out.push_str("  )\n");
    out.push_str("})\n");
    out.push_str("]\n");
    out
}

/// Defects found on acceptance (work order).
fn damages_block_markup(data: &DocumentData) -> String {
    let text = data
        .condition_report
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("");
    ruled_text_block_markup(
        "При отриманні транспортного засобу були знайдені наступні існуючі дефекти:",
        text,
    )
}

/// General recommendations and the defects whose repair the customer refused
/// (acceptance act). Rendered only when the field is filled in.
fn refused_defects_markup(text: &str) -> String {
    ruled_text_block_markup(
        "Загальні рекомендації, та дефекти, від усунення яких замовник відмовився:",
        text,
    )
}

// ---------------------------------------------------------------------------
// Signatures
// ---------------------------------------------------------------------------

fn signatures_markup(data: &DocumentData) -> String {
    let mut out = String::new();
    // The signature block does not break across pages — otherwise the
    // performer's signature could end up on one page and the customer's on
    // another.
    out.push_str(
        "#v(1em)\n#block(breakable: false)[\n#grid(\n  columns: (1fr, 1fr),\n  column-gutter: 2.5em,\n  row-gutter: 1em,\n",
    );

    match data.kind {
        DocumentKind::NaryadZamovlennya => {
            out.push_str("  [\n    #text(weight: \"bold\", style: \"italic\")[Від виконавця (або Представника)] \\\n    #v(0.25em)\n    #text(style: \"italic\", size: 0.85em, fill: rgb(\"#5B6772\"))[Транспортний засіб прийняв, замовлення відкрив]\n  ],\n");
            out.push_str("  [\n    #text(weight: \"bold\", style: \"italic\")[Від замовника (або Представника)] \\\n    #v(0.25em)\n    #text(style: \"italic\", size: 0.85em, fill: rgb(\"#5B6772\"))[Транспортний засіб здав, з умовами виконання замовлення та правилами автосервісу ознайомлений та зобов'язуюсь їх виконувати]\n  ],\n");
        }
        DocumentKind::AktVykonanykhRobit => {
            out.push_str("  text(weight: \"bold\", style: \"italic\")[Від виконавця (або Представника)\\*],\n");
            out.push_str("  [\n    #text(weight: \"bold\", style: \"italic\")[Від замовника (або Представника)] \\\n    #v(0.25em)\n    #text(style: \"italic\", size: 0.85em, fill: rgb(\"#5B6772\"))[Транспортний засіб отримав. Роботи виконані в повному обсязі та належної якості. З вартістю та переліком робіт згоден. Претензій щодо якості послуг, комплектності та зовнішнього вигляду транспортного засобу не маю.]\n  ],\n");
        }
    }

    out.push_str("  grid(\n    columns: (1fr, 8em),\n    column-gutter: 1em,\n");
    out.push_str(&format!(
        "    sig-field(value: [{}], label: \"посада, прізвище, ім'я, по батькові\"),\n",
        esc(&data.performer_signature)
    ));
    out.push_str("    sig-field(label: \"підпис\"),\n  ),\n");

    out.push_str("  grid(\n    columns: (1fr, 8em),\n    column-gutter: 1em,\n");
    out.push_str(&format!(
        "    sig-field(value: [{}], label: \"прізвище, ім'я, по батькові\"),\n",
        esc(&data.customer_signature)
    ));
    out.push_str("    sig-field(label: \"підпис\"),\n  ),\n");

    out.push_str(")\n");

    if data.kind == DocumentKind::AktVykonanykhRobit {
        out.push_str("#v(1em)\n#text(\n  size: 8pt,\n");
        out.push_str(&format!("  fill: {CLR_MUTED},\n  style: \"italic\",\n"));
        out.push_str("  [\\* Відповідальний за здійснення господарської операції та правильність її оформлення],\n)\n");
    }
    out.push_str("]\n");
    out
}

// ---------------------------------------------------------------------------
// Service rules
// ---------------------------------------------------------------------------

fn rules_markup() -> String {
    let mut out = String::new();
    out.push_str("#v(0.8em)\n");
    out.push_str(&format!(
        "#block(\n  width: 100%,\n  fill: {CLR_BG_RULES},\n  stroke: {STROKE_REG},\n  radius: 2pt,\n  inset: (x: 1.1em, y: 0.8em),\n  breakable: false,\n)[\n"
    ));
    out.push_str(&format!(
        "  #text(weight: \"bold\", size: 9.5pt, fill: {CLR_ACCENT})[Правила автосервісу]\n"
    ));
    out.push_str("  #v(0.6em)\n");
    out.push_str("  #set text(size: 8pt)\n");
    out.push_str("  #set par(spacing: 0.4em, leading: 0.42em)\n");
    out.push_str("  #enum(\n    indent: 0pt,\n    body-indent: 0.4em,\n    spacing: 0.45em,\n");
    out.push_str(
        "    [Автосервіс не несе відповідальності за збереження речей, залишених в автомобілі.],\n",
    );
    out.push_str("    [Автосервіс несе відповідальність тільки за оплачені у наряд-замовленні роботи та запчастини.],\n");
    out.push_str("    [Автосервіс залишає за собою право на продовження термінів ремонту.],\n");
    out.push_str("    [З метою контролю якості виконаних робіт, співробітник(и) можуть самостійно тестувати автомобіль, у тому числі за межами автосервісу.],\n");
    out.push_str("  )\n");
    out.push_str("  #v(0.6em)\n");
    out.push_str("  Узгоджений наряд-замовлення слугує підставою для початку ремонту.\n");
    out.push_str("]\n");
    out
}

// ---------------------------------------------------------------------------
// Document assembly
// ---------------------------------------------------------------------------

/// Document prelude: `shrink`, page, fonts and helper functions.
///
/// `shrink` is defined before `#set page`, because the footer captures the
/// visible scope at the moment the `set` rule runs.
fn document_prelude(footer_label: &str) -> String {
    let mut out = String::new();

    // Shrinks an over-long unbreakable token (IBAN, VIN, code) without breaking
    // the line. `layout` and `scale` are no good for this: typst treats them as
    // block-level, so they move the token onto a separate line and add
    // inter-block spacing even when no shrinking is needed.
    // `context` + `measure` + a smaller font size stay inside the line.
    //
    // `measure` is called without `width`, i.e. at unbounded width: with a
    // limit it returns the already wrapped (and therefore always fitting)
    // width, and overflow becomes impossible to detect.
    out.push_str("#let shrink(limit, body) = context {\n");
    out.push_str("  let w = measure(body).width\n");
    out.push_str("  if w <= limit { body } else {\n");
    out.push_str("    text(size: text.size * (limit / w), body)\n");
    out.push_str("  }\n");
    out.push_str("}\n");

    out.push_str("#set page(\n  paper: \"a4\",\n  margin: (x: 1cm, top: 1cm, bottom: 1.3cm),\n  footer: context [\n");
    out.push_str(&format!("    #set text(size: 7.5pt, fill: {CLR_MUTED})\n"));
    out.push_str(&format!("    #line(length: 100%, stroke: {STROKE_THIN})\n"));
    out.push_str("    #v(0.25em)\n");
    out.push_str("    #grid(\n      columns: (1fr, 1fr),\n      align: (left, right),\n");
    out.push_str(&format!(
        "      [{}],\n      [Сторінка #counter(page).display() з #context counter(page).final().first()],\n",
        esc(footer_label)
    ));
    out.push_str("    )\n  ],\n)\n");
    out.push_str("#set text(font: (\"IBM Plex Sans\", \"Libertinus Serif\", \"New Computer Modern\", \"DejaVu Sans Mono\"), size: 9.5pt, lang: \"uk\", fill: rgb(\"#101418\"), hyphenate: true)\n");
    out.push_str("#set par(leading: 0.55em, spacing: 0.65em, justify: false)\n\n");
    out.push_str("#let sig-field(value: [], label: \"\", width: 100%) = block(width: width)[\n");
    out.push_str(
        "  #block(height: 1.4em, width: 100%, align(bottom + left)[#text(size: 0.92em)[#value]])\n",
    );
    out.push_str(&format!("  #line(length: 100%, stroke: {STROKE_REG})\n"));
    out.push_str("  #v(0.5pt)\n");
    out.push_str(&format!(
        "  #align(left)[#text(size: 0.74em, fill: {CLR_MUTED}, style: \"italic\")[#label]]\n"
    ));
    out.push_str("]\n\n");

    out
}

fn build_markup(data: &DocumentData) -> String {
    let footer_label = match data.kind {
        DocumentKind::NaryadZamovlennya => format!("Наряд-замовлення № {}", data.work_order_id),
        DocumentKind::AktVykonanykhRobit => format!("Акт виконаних робіт № {}", data.work_order_id),
    };

    let mut out = document_prelude(&footer_label);

    // Header.
    out.push_str(&business_header_markup(data));
    out.push_str("#v(0.6em)\n");

    // Title.
    out.push_str(&title_block_markup(data));
    out.push_str("#v(0.6em)\n");

    // Car and clients.
    out.push_str(&info_grid_markup(data));
    out.push_str("#v(0.6em)\n");

    // Work order: time frames
    if let Some(terms) = terms_block_markup(data) {
        out.push_str(&terms);
        out.push_str("#v(0.6em)\n");
    }

    // Services and parts table
    out.push_str("#set text(size: 8pt)\n");
    out.push_str(&items_table_markup(data.kind, &data.labor, &data.parts));
    out.push_str("#v(0.8em)\n");
    out.push_str("#set text(size: 9.5pt)\n");

    // Totals and the amount in words.
    out.push_str(&totals_markup(data));
    out.push_str(&format!(
        "#v(0.4em)\n#align(right)[\n  #text(size: 10pt, fill: {CLR_MUTED}, style: \"italic\")[({})]\n]\n",
        esc(&data.total_in_words)
    ));

    // Clauses
    match data.kind {
        DocumentKind::NaryadZamovlennya => {
            out.push_str("#v(0.8em)\n");
            out.push_str(&old_parts_clause_markup());
        }
        DocumentKind::AktVykonanykhRobit => {
            if let Some(refused) = data
                .refused_defects
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                out.push_str("#v(0.8em)\n");
                out.push_str(&refused_defects_markup(refused));
            }
        }
    }

    // Work order: defects.
    if data.kind == DocumentKind::NaryadZamovlennya {
        out.push_str("#v(1em)\n");
        out.push_str(&damages_block_markup(data));
    }

    // Signatures.
    out.push_str(&signatures_markup(data));

    // Work order: service rules.
    if data.kind == DocumentKind::NaryadZamovlennya {
        out.push_str(&rules_markup());
    }

    out
}

// ---------------------------------------------------------------------------
// Typst world with embedded fonts
// ---------------------------------------------------------------------------

struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    source: Source,
    main_id: FileId,
}

impl TypstWorld {
    fn new(markup: String) -> Self {
        // IBM Plex Sans first, then fallback fonts
        let mut all_fonts: Vec<&[u8]> = EMBEDDED_FONTS.to_vec();
        all_fonts.extend(typst_assets::fonts());
        let fonts: Vec<Font> = all_fonts
            .into_iter()
            .filter_map(|bytes| Font::new(Bytes::new(bytes.to_vec()), 0))
            .collect();
        let book = FontBook::from_fonts(&fonts);

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            source: Source::detached(markup),
            main_id: RootedPath::new(
                VirtualRoot::Project,
                VirtualPath::new("/main.typ").expect("valid virtual path"),
            )
            .intern(),
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id.vpath().get_without_slash() == "main.typ" {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(PathBuf::new()))
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(PathBuf::new()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<typst::foundations::Datetime> {
        None
    }
}

fn layout_document(data: &DocumentData) -> AppResult<PagedDocument> {
    let markup = build_markup(data);
    let world = TypstWorld::new(markup);

    let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);
    for warning in warnings.iter() {
        log::warn!("Typst warning: {}", warning.message);
    }

    output.map_err(|errors| {
        let messages = errors
            .iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        AppError::Generic(format!("Не вдалося згенерувати PDF: {messages}"))
    })
}

pub fn render(data: &DocumentData) -> AppResult<Vec<u8>> {
    let document = layout_document(data)?;

    let bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося згенерувати PDF: {:?}",
            e.iter().collect::<Vec<_>>()
        ))
    })?;

    Ok(bytes)
}
