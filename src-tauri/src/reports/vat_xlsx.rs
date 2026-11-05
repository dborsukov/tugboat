//! VAT report workbook renderer.

use chrono::Utc;
use rust_xlsxwriter::{
    Color, DocProperties, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError,
};
use tauri::State;

use super::{
    figures::ReportContext,
    structs::DayRange,
    vat_figures::VatReportFigures,
    xlsx::{excel_day, fraction, money},
};
use crate::{database::vat::structs::VatEntry, error::AppResult, utils::plural, AppState};

const SHEET_SUMMARY: &str = "Зведення";
const SHEET_REGISTER: &str = "Реєстр";

const FOOTER: &str = "&L&\"Arial,Regular\"&8 Реєстр ПДВ&C&\"Arial,Regular\"&8 Сторінка &P з &N&R&\"Arial,Regular\"&8 &D";

const NUMBER_MONEY: &str = "#,##0.00;\\-#,##0.00;\\–";
const NUMBER_DATE: &str = "dd\\.mm\\.yyyy";
const NUMBER_PERCENT_0: &str = "0%;\\-0%;\\–";
const NUMBER_INTEGER: &str = "#,##0";

const NAVY: u32 = 0x1F3A5F;
const BLUE: u32 = 0x2A78D6;
const DARK: u32 = 0x1F2937;
const GRAY: u32 = 0x6B7280;
const CARD: u32 = 0xEEF3FA;
const BORDER: u32 = 0xE5E7EB;
const WHITE: u32 = 0xFFFFFF;
const LIGHT_BLUE: u32 = 0xC7D5EA;
const REVERSAL_FG: u32 = 0xB91C1C;

const SUMMARY_WIDTHS: [f64; 8] = [2.0, 18.0, 18.0, 18.0, 18.0, 18.0, 18.0, 2.0];

const FIRST_COL: u16 = 1;
const LAST_COL: u16 = 6;

const NOTE_HEIGHT: f64 = 13.5;

const NOTES: [&str; 4] = [
    "Податкові зобов'язання зафіксовано в момент податкової події: дата виконання робіт або дата оплати, залежно від того, що настало раніше.",
    "Податковий кредит наведено за датою податкової накладної постачальника. Кредитуються лише витрати, позначені як включені до податкового кредиту.",
    "Сторно-записи наведено окремими рядками з від'ємними сумами; вони зменшують той бік, до якого належать, у періоді, коли зроблені.",
    "Закупівля запчастин, не оформлена як витрата з податковою накладною, у кредиті не наведена.",
];

struct Formats {
    banner_title: Format,
    banner_subtitle: Format,
    title: Format,
    subtitle: Format,
    section: Format,
    kpi_label: Format,
    kpi_value: Format,
    kpi_value_blue: Format,
    kpi_caption: Format,
    notes_heading: Format,
    header_left: Format,
    header_center: Format,
    header_right: Format,
    text: Format,
    text_center: Format,
    date: Format,
    code: Format,
    money: Format,
    rate: Format,
    integer: Format,
    total_label: Format,
    total_money: Format,
    note: Format,
    kind_output: Format,
    kind_input: Format,
    kind_reversal: Format,
}

impl Formats {
    fn new() -> Self {
        let base = |size: u8| Format::new().set_font_name("Arial").set_font_size(size);
        let row = |format: Format| {
            format
                .set_border_bottom(FormatBorder::Thin)
                .set_border_bottom_color(Color::RGB(BORDER))
        };
        // A headline card is one pale block split from its neighbours by white
        // rules, the way the financial report draws its four figures.
        let card = |format: Format| {
            format
                .set_background_color(Color::RGB(CARD))
                .set_border_left(FormatBorder::Medium)
                .set_border_left_color(Color::RGB(WHITE))
                .set_border_right(FormatBorder::Medium)
                .set_border_right_color(Color::RGB(WHITE))
        };

        Self {
            banner_title: base(18)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            banner_subtitle: base(9)
                .set_font_color(Color::RGB(LIGHT_BLUE))
                .set_background_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            title: base(15)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_align(FormatAlign::VerticalCenter),
            subtitle: base(9).set_font_color(Color::RGB(GRAY)),
            section: base(11)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            header_left: base(9)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_text_wrap()
                .set_indent(1),
            header_center: base(9)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_text_wrap(),
            header_right: base(9)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_text_wrap()
                .set_indent(1),
            text: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_align(FormatAlign::Left)
                .set_indent(1),
            text_center: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_align(FormatAlign::Center),
            date: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_DATE)
                .set_align(FormatAlign::Center),
            code: row(base(10).set_font_color(Color::RGB(DARK))).set_align(FormatAlign::Center),
            money: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_indent(1),
            rate: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_PERCENT_0)
                .set_align(FormatAlign::Center),
            integer: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_INTEGER)
                .set_align(FormatAlign::Center),
            total_label: base(10)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_background_color(Color::RGB(CARD))
                .set_align(FormatAlign::Left)
                .set_indent(1),
            total_money: base(10)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_background_color(Color::RGB(CARD))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_indent(1),
            kpi_label: card(base(8).set_bold().set_font_color(Color::RGB(GRAY)))
                .set_border_top(FormatBorder::Medium)
                .set_border_top_color(Color::RGB(BLUE))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::Bottom)
                .set_indent(1),
            kpi_value: card(base(18).set_bold().set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            kpi_value_blue: card(base(18).set_bold().set_font_color(Color::RGB(BLUE)))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            kpi_caption: card(base(8).set_font_color(Color::RGB(GRAY)))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::Top)
                .set_indent(1),
            notes_heading: base(10).set_bold().set_font_color(Color::RGB(NAVY)),
            note: base(9)
                .set_font_color(Color::RGB(GRAY))
                .set_text_wrap()
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::Top),
            kind_output: row(base(10).set_font_color(Color::RGB(DARK)))
                .set_align(FormatAlign::Left)
                .set_indent(1),
            kind_input: row(base(10).set_font_color(Color::RGB(BLUE)))
                .set_align(FormatAlign::Left)
                .set_indent(1),
            kind_reversal: row(base(10).set_bold().set_font_color(Color::RGB(REVERSAL_FG)))
                .set_align(FormatAlign::Left)
                .set_indent(1),
        }
    }
}

fn page_setup(worksheet: &mut Worksheet) {
    worksheet
        .set_landscape()
        .set_paper_size(9)
        .set_margins(0.5, 0.5, 0.6, 0.7, 0.51, 0.5)
        .set_print_center_horizontally(true)
        .set_print_fit_to_pages(1, 0)
        .set_footer(FOOTER);
}

fn draw_summary(
    worksheet: &mut Worksheet,
    formats: &Formats,
    report: &VatReportFigures,
) -> Result<(), XlsxError> {
    let f = formats;
    worksheet.set_name(SHEET_SUMMARY)?;
    worksheet.set_screen_gridlines(false);

    for (col, width) in SUMMARY_WIDTHS.iter().enumerate() {
        worksheet.set_column_width(col as u16, *width)?;
    }

    // Banner, then the three headline figures, the same opening as the
    // financial report: the title on navy, the period under it, then the cards.
    for (row, height) in [
        (0u32, 37.5),
        (1, 24.0),
        (2, 12.0),
        (3, 21.75),
        (4, 39.75),
        (5, 21.75),
        (6, 13.5),
    ] {
        worksheet.set_row_height(row, height)?;
    }

    worksheet.merge_range(0, FIRST_COL, 0, LAST_COL, "Реєстр ПДВ", &f.banner_title)?;
    worksheet.merge_range(
        1,
        FIRST_COL,
        1,
        LAST_COL,
        &report.subtitle,
        &f.banner_subtitle,
    )?;

    let labels = [
        "ПДВ НАРАХОВАНО, ГРН",
        "ПОДАТКОВИЙ КРЕДИТ, ГРН",
        "ПДВ ДО СПЛАТИ, ГРН",
    ];
    let values = [
        money(report.output.amount),
        money(report.input.amount),
        money(report.net),
    ];
    let captions = [
        entries_caption(report.output.entries),
        entries_caption(report.input.entries),
        // A negative figure is money the period is owed back, which the caption
        // names rather than leaving the sign to speak for itself.
        if report.net.is_sign_negative() {
            "до відшкодування".to_string()
        } else {
            "нараховано − кредит".to_string()
        },
    ];

    // Three cards of two columns each: they cover the content width exactly.
    for (index, col) in (FIRST_COL..=LAST_COL).step_by(2).enumerate() {
        worksheet.merge_range(3, col, 3, col + 1, labels[index], &f.kpi_label)?;
        let value_format = if index == 2 {
            &f.kpi_value_blue
        } else {
            &f.kpi_value
        };
        merge_number(worksheet, 4, col, 4, col + 1, values[index], value_format)?;
        worksheet.merge_range(5, col, 5, col + 1, &captions[index], &f.kpi_caption)?;
    }

    let mut row = 7u32;

    worksheet.set_row_height(row, 24.0)?;
    worksheet.merge_range(row, FIRST_COL, row, LAST_COL, "За ставками", &f.section)?;

    row += 1;
    worksheet.set_row_height(row, 25.5)?;
    worksheet.merge_range(row, FIRST_COL, row, 2, "Ставка", &f.header_center)?;
    worksheet.write_string_with_format(row, 3, "База нараховано", &f.header_right)?;
    worksheet.write_string_with_format(row, 4, "ПДВ нараховано", &f.header_right)?;
    worksheet.write_string_with_format(row, 5, "База кредит", &f.header_right)?;
    worksheet.write_string_with_format(row, LAST_COL, "ПДВ кредит", &f.header_right)?;

    for rate in &report.by_rate {
        row += 1;
        worksheet.set_row_height(row, 19.5)?;

        let label = if rate.scheme.is_standard() {
            format!("{} %", rate.rate.normalize())
        } else {
            format!("{} % · {}", rate.rate.normalize(), rate.scheme.label())
        };
        worksheet.merge_range(row, FIRST_COL, row, 2, &label, &f.text_center)?;
        worksheet.write_number_with_format(row, 3, money(rate.output_base), &f.money)?;
        worksheet.write_number_with_format(row, 4, money(rate.output_amount), &f.money)?;
        worksheet.write_number_with_format(row, 5, money(rate.input_base), &f.money)?;
        worksheet.write_number_with_format(row, LAST_COL, money(rate.input_amount), &f.money)?;
    }

    row += 1;
    worksheet.set_row_height(row, 21.75)?;
    worksheet.merge_range(row, FIRST_COL, row, 2, "Разом", &f.total_label)?;
    worksheet.write_number_with_format(row, 3, money(report.output.base), &f.total_money)?;
    worksheet.write_number_with_format(row, 4, money(report.output.amount), &f.total_money)?;
    worksheet.write_number_with_format(row, 5, money(report.input.base), &f.total_money)?;
    worksheet.write_number_with_format(
        row,
        LAST_COL,
        money(report.input.amount),
        &f.total_money,
    )?;

    row += 1;
    worksheet.set_row_height(row, 12.0)?;

    // The month table starts on the same column and, with its label merged over
    // the rate label and the first figure, ends on the same column too.
    row += 1;
    worksheet.set_row_height(row, 24.0)?;
    worksheet.merge_range(row, FIRST_COL, row, LAST_COL, "За місяцями", &f.section)?;

    row += 1;
    worksheet.set_row_height(row, 25.5)?;
    worksheet.merge_range(row, FIRST_COL, row, 3, "Місяць", &f.header_left)?;
    worksheet.write_string_with_format(row, 4, "Нараховано", &f.header_right)?;
    worksheet.write_string_with_format(row, 5, "Кредит", &f.header_right)?;
    worksheet.write_string_with_format(row, LAST_COL, "До сплати", &f.header_right)?;

    for month in &report.by_month {
        row += 1;
        worksheet.set_row_height(row, 19.5)?;

        worksheet.merge_range(row, FIRST_COL, row, 3, &month.label, &f.text)?;
        worksheet.write_number_with_format(row, 4, money(month.output_amount), &f.money)?;
        worksheet.write_number_with_format(row, 5, money(month.input_amount), &f.money)?;
        worksheet.write_number_with_format(row, LAST_COL, money(month.net()), &f.money)?;
    }

    row += 1;
    worksheet.set_row_height(row, 12.0)?;

    // Every note fits on one line across the merged width, so each takes a
    // single tight row rather than the tall rows it needed when it wrapped.
    row += 1;
    worksheet.set_row_height(row, 18.0)?;
    worksheet.write_string_with_format(row, FIRST_COL, "Примітки", &f.notes_heading)?;

    row += 1;
    for note in NOTES {
        worksheet.set_row_height(row, NOTE_HEIGHT)?;
        worksheet.merge_range(row, FIRST_COL, row, LAST_COL, note, &f.note)?;
        row += 1;
    }

    page_setup(worksheet);
    Ok(())
}

/// A merged range holding one number, the card the value is drawn on.
fn merge_number(
    worksheet: &mut Worksheet,
    first_row: u32,
    first_col: u16,
    last_row: u32,
    last_col: u16,
    value: f64,
    format: &Format,
) -> Result<(), XlsxError> {
    worksheet.merge_range(first_row, first_col, last_row, last_col, "", format)?;
    worksheet.write_number_with_format(first_row, first_col, value, format)?;
    Ok(())
}

/// The line under a headline figure, e.g. "8 записів".
fn entries_caption(count: usize) -> String {
    format!("{} {}", count, plural(count, "запис", "записи", "записів"))
}

fn draw_register(
    worksheet: &mut Worksheet,
    formats: &Formats,
    report: &VatReportFigures,
) -> Result<(), XlsxError> {
    let f = formats;
    worksheet.set_name(SHEET_REGISTER)?;
    worksheet.set_screen_gridlines(false);

    let widths = [
        4.0, 14.0, 14.0, 18.0, 18.0, 32.0, 14.0, 16.0, 10.0, 16.0, 18.0,
    ];
    for (col, width) in widths.iter().enumerate() {
        worksheet.set_column_width(col as u16, *width)?;
    }
    worksheet.set_row_height(0, 26.0)?;
    worksheet.set_row_height(1, 16.0)?;
    worksheet.set_row_height(2, 8.0)?;
    worksheet.set_row_height(3, 32.0)?;

    worksheet.merge_range(0, 0, 0, 10, "Реєстр ПДВ — зобов'язання та кредит", &f.title)?;
    worksheet.merge_range(1, 0, 1, 10, &report.subtitle, &f.subtitle)?;

    let headers: [(&str, &Format); 11] = [
        ("№", &f.header_center),
        ("Дата податкової події", &f.header_center),
        ("Вид", &f.header_left),
        ("Документ", &f.header_left),
        ("Замовлення / витрата", &f.header_center),
        ("Контрагент", &f.header_left),
        ("Код", &f.header_center),
        ("База, грн", &f.header_right),
        ("Ставка", &f.header_center),
        ("ПДВ, грн", &f.header_right),
        ("Примітка", &f.header_left),
    ];
    for (col, (text, format)) in headers.into_iter().enumerate() {
        worksheet.write_string_with_format(3, col as u16, text, format)?;
    }
    // Repeat the register's header row on every printed page.
    worksheet.set_repeat_rows(3, 3)?;

    for (index, entry) in report.entries.iter().enumerate() {
        let row = 4 + index as u32;
        worksheet.set_row_height(row, 19.5)?;

        worksheet.write_number_with_format(row, 0, (index + 1) as f64, &f.integer)?;
        match entry.tax_point {
            Some(day) => worksheet.write_number_with_format(row, 1, excel_day(day), &f.date)?,
            // A stored date that no longer parses is printed as it stands rather
            // than silently moved to the first day of the period.
            None => worksheet.write_string_with_format(row, 1, &entry.tax_point_text, &f.text)?,
        };
        let kind_format = if entry.reversal {
            &f.kind_reversal
        } else {
            match entry.direction {
                crate::database::vat::structs::VatDirection::Output => &f.kind_output,
                crate::database::vat::structs::VatDirection::Input => &f.kind_input,
            }
        };
        worksheet.write_string_with_format(row, 2, entry.kind(), kind_format)?;
        worksheet.write_string_with_format(row, 3, &entry.document, &f.text)?;
        worksheet.write_string_with_format(row, 4, &entry.source_ref, &f.code)?;
        worksheet.write_string_with_format(row, 5, &entry.counterparty, &f.text)?;
        worksheet.write_string_with_format(row, 6, &entry.code, &f.code)?;
        worksheet.write_number_with_format(row, 7, money(entry.base), &f.money)?;
        worksheet.write_number_with_format(
            row,
            8,
            fraction(entry.rate / rust_decimal::Decimal::ONE_HUNDRED),
            &f.rate,
        )?;
        worksheet.write_number_with_format(row, 9, money(entry.amount), &f.money)?;
        let mut note_parts: Vec<String> = Vec::new();
        if !entry.scheme_label().is_empty() {
            note_parts.push(entry.scheme_label().to_string());
        }
        if let Some(code) = &entry.exemption_code {
            note_parts.push(format!("Пільга {code}"));
        }
        let note = note_parts.join(" · ");
        worksheet.write_string_with_format(row, 10, note, &f.text)?;
    }

    let mut totals_row = 4 + report.entries.len() as u32;

    worksheet.set_row_height(totals_row, 22.0)?;
    worksheet.merge_range(
        totals_row,
        0,
        totals_row,
        6,
        "ПДВ нараховано",
        &f.total_label,
    )?;
    worksheet.write_number_with_format(totals_row, 7, money(report.output.base), &f.total_money)?;
    worksheet.write_blank(totals_row, 8, &f.total_money)?;
    worksheet.write_number_with_format(
        totals_row,
        9,
        money(report.output.amount),
        &f.total_money,
    )?;
    worksheet.write_string_with_format(totals_row, 10, "", &f.total_label)?;

    totals_row += 1;
    worksheet.set_row_height(totals_row, 22.0)?;
    worksheet.merge_range(
        totals_row,
        0,
        totals_row,
        6,
        "Податковий кредит",
        &f.total_label,
    )?;
    worksheet.write_number_with_format(totals_row, 7, money(report.input.base), &f.total_money)?;
    worksheet.write_blank(totals_row, 8, &f.total_money)?;
    worksheet.write_number_with_format(
        totals_row,
        9,
        money(report.input.amount),
        &f.total_money,
    )?;
    worksheet.write_string_with_format(totals_row, 10, "", &f.total_label)?;

    totals_row += 1;
    worksheet.set_row_height(totals_row, 22.0)?;
    worksheet.merge_range(totals_row, 0, totals_row, 6, "До сплати", &f.total_label)?;
    worksheet.write_number_with_format(totals_row, 7, money(report.net_base), &f.total_money)?;
    worksheet.write_blank(totals_row, 8, &f.total_money)?;
    worksheet.write_number_with_format(totals_row, 9, money(report.net), &f.total_money)?;
    worksheet.write_string_with_format(totals_row, 10, "", &f.total_label)?;

    page_setup(worksheet);
    Ok(())
}

fn build_workbook(report: &VatReportFigures) -> Result<Workbook, XlsxError> {
    let formats = Formats::new();

    let mut workbook = Workbook::new();
    let properties = DocProperties::new()
        .set_title(format!(
            "Реєстр ПДВ {} – {}",
            report.period.start.format("%d.%m.%Y"),
            report.period.end.format("%d.%m.%Y")
        ))
        .set_subject("Податкові зобов'язання та податковий кредит за період")
        .set_company(&report.company)
        .set_author(&report.company);
    workbook.set_properties(&properties);

    {
        let worksheet = workbook.add_worksheet();
        draw_summary(worksheet, &formats, report)?;
    }
    {
        let worksheet = workbook.add_worksheet();
        draw_register(worksheet, &formats, report)?;
    }

    Ok(workbook)
}

pub async fn render(
    state: &State<'_, AppState>,
    range: DayRange,
    entries: &[VatEntry],
) -> AppResult<Vec<u8>> {
    let settings = crate::database::settings::fetch_settings_from_pool(&state.db_pool).await?;
    let context = ReportContext::from_settings(&settings, Utc::now());
    let report = VatReportFigures::new(&context, range, entries);

    Ok(build_workbook(&report)?.save_to_buffer()?)
}
