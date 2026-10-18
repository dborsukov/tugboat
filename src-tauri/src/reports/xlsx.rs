//! Financial report workbook renderer.
//!
//! Every value is computed in [`super::figures`] and written as a number, so
//! the file opens with the same figures the dashboard shows and needs no
//! recalculation. Only conditional formatting is delegated to Excel.

use chrono::{NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_xlsxwriter::{
    Chart, ChartDataLabel, ChartDataLabelPosition, ChartFont, ChartFormat, ChartLine,
    ChartSolidFill, ChartType, Color, ConditionalFormatDataBar, ConditionalFormatFormula,
    ConditionalFormatType, DocProperties, Format, FormatAlign, FormatBorder, Note, Workbook,
    Worksheet, XlsxError,
};
use tauri::State;

use super::{
    figures::{ReportContext, ReportFigures, WaterfallKind},
    hash::ReportSources,
    structs::{DayRange, ReportKind},
};
use crate::{
    error::{AppError, AppResult},
    AppState,
};

const SHEET_SUMMARY: &str = "Зведення";
const SHEET_ORDERS: &str = "Замовлення";
const SHEET_EXPENSES: &str = "Витрати";
const SHEET_SERVICE: &str = "Службові дані";

const FOOTER: &str = "&L&\"Arial,Regular\"&8 Фінансовий звіт&C&\"Arial,Regular\"&8 Сторінка &P з &N&R&\"Arial,Regular\"&8 &D";

const NOTE_MISSING_PARTS_COST: &str = "Закупівельну ціну запчастин не вказано в замовленні. Валовий дохід по цьому замовленню завищено.";
const NOTE_MISSING_LABOR_COST: &str = "Собівартість робіт не визначено: для погодинної собівартості в замовленні є роботи з фіксованою ціною. Валовий дохід по цьому замовленню завищено.";
const NOTE_AUTHOR: &str = "Папай CRM";

const ORDERS_NOTE: &str = "Собівартість роботи — вартість робіт за нарядом, запчастин — закупівельна ціна. Якщо собівартість не вказано, валовий дохід по замовленню завищено (позначено жовтим).";
const ORDERS_TITLE: &str = "Замовлення, оплачені за період";
const EXPENSES_TITLE: &str = "Витрати поза замовленнями";

const NOTES: [&str; 3] = [
    "Усі суми вказано в гривнях. Для ПДВ-платників суми наведено без ПДВ.",
    "Собівартість роботи — вартість робіт за нарядом, запчастин — закупівельна ціна з замовлень. Витрати поза замовленнями — з розділу «Витрати».",
    "Валовий дохід за напрямами: послуги — робота після собівартості, запчастини — після закупівельної ціни; знижки розподілено пропорційно виручці.",
];
const NOTE_GENERATED: &str = "Дані наведено станом на дату формування звіту.";

// Number formats. The dash form is the design's way of saying "nothing here",
// so a zero never looks like a figure.
const NUMBER_MONEY: &str = "#,##0.00;\\-#,##0.00;\\–";
const NUMBER_MONEY_RED: &str = "#,##0.00;[RED]\\-#,##0.00;\\–";
const NUMBER_PERCENT_1: &str = "0.0%;\\-0.0%;\\–";
const NUMBER_PERCENT_0: &str = "0%;\\-0%;\\–";
const NUMBER_DATE: &str = "dd\\.mm\\.yyyy";
const NUMBER_INTEGER: &str = "#,##0";

const NAVY: u32 = 0x1F3A5F;
const BLUE: u32 = 0x2A78D6;
const DARK: u32 = 0x1F2937;
const GRAY: u32 = 0x6B7280;
const LIGHT_BLUE: u32 = 0xC7D5EA;
const CARD: u32 = 0xEEF3FA;
const BAND: u32 = 0xF6F8FB;
const GROSS_BG: u32 = 0xDCE9FB;
const BORDER: u32 = 0xE5E7EB;
const WHITE: u32 = 0xFFFFFF;
const WARN_FG: u32 = 0xB45309;
const WARN_BG: u32 = 0xFEF3C7;
const DATA_BAR: u32 = 0xB7D3F5;
const CHART_LIGHT: u32 = 0x9DB4D3;
const CHART_AXIS: u32 = 0x878787;
const CHART_BORDER: u32 = 0xD9D9D9;

const SUMMARY_WIDTHS: [f64; 10] = [2.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
const SUMMARY_HEIGHTS: [(u32, f64); 26] = [
    (0, 37.5),
    (1, 24.0),
    (2, 12.0),
    (3, 21.75),
    (4, 39.75),
    (5, 21.75),
    (6, 13.5),
    (7, 24.0),
    (8, 24.0),
    (9, 21.75),
    (10, 21.75),
    (11, 21.75),
    (12, 21.75),
    (13, 21.75),
    (14, 21.75),
    (15, 21.75),
    (16, 21.75),
    (17, 21.75),
    (18, 21.75),
    (19, 21.75),
    (20, 12.0),
    (21, 18.0),
    // Every note fits on one line across the merged width.
    (22, 13.5),
    (23, 13.5),
    (24, 13.5),
    (25, 13.5),
];

const ORDERS_WIDTHS: [f64; 13] = [
    15.0, 13.0, 22.0, 17.0, 18.0, 13.0, 14.0, 12.0, 14.0, 15.0, 17.0, 15.0, 10.0,
];
const EXPENSES_WIDTHS: [f64; 6] = [13.0, 32.0, 22.0, 14.0, 10.0, 22.0];

const SERVICE_WIDTHS: [f64; 3] = [46.0, 16.0, 40.0];

/// The formats the sheets share, built once per workbook.
struct Formats {
    banner_title: Format,
    banner_subtitle: Format,
    kpi_label: Format,
    kpi_value: Format,
    kpi_value_blue: Format,
    kpi_margin: Format,
    kpi_net: Format,
    kpi_caption: Format,
    section: Format,
    waterfall_header_left: Format,
    waterfall_header_right: Format,
    waterfall_label: Format,
    waterfall_money: Format,
    waterfall_percent: Format,
    subtotal_label: Format,
    subtotal_money: Format,
    subtotal_percent: Format,
    gross_label: Format,
    gross_money: Format,
    gross_percent: Format,
    final_label: Format,
    final_money: Format,
    final_percent: Format,
    breakdown_label: Format,
    breakdown_money: Format,
    breakdown_percent: Format,
    notes_heading: Format,
    note: Format,
    note_italic: Format,
    sheet_title: Format,
    sheet_subtitle: Format,
    orders_header_left: Format,
    orders_header_center: Format,
    orders_header_right: Format,
    order_text: Format,
    order_date: Format,
    order_money: Format,
    order_percent: Format,
    order_text_banded: Format,
    order_date_banded: Format,
    order_money_banded: Format,
    order_percent_banded: Format,
    orders_total_label: Format,
    orders_total_money: Format,
    orders_total_percent: Format,
    orders_note: Format,
    missing_cost: Format,
    expenses_header_center: Format,
    expenses_header_left: Format,
    expenses_header_right: Format,
    expense_vat: Format,
    expense_vat_banded: Format,
    expenses_total_label: Format,
    expenses_total_empty: Format,
    categories_blank: Format,
    categories_blank_banded: Format,
    service_label: Format,
    service_number: Format,
}

impl Formats {
    fn new() -> Self {
        let base = |size: u8| Format::new().set_font_name("Arial").set_font_size(size);
        let card_borders = |format: Format| {
            format
                .set_border_left(FormatBorder::Medium)
                .set_border_left_color(Color::RGB(WHITE))
                .set_border_right(FormatBorder::Medium)
                .set_border_right_color(Color::RGB(WHITE))
        };
        let row_border = |format: Format| {
            format
                .set_border_bottom(FormatBorder::Thin)
                .set_border_bottom_color(Color::RGB(BORDER))
        };
        let table_header = |format: Format| {
            format
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
        };
        let pale_total = |format: Format| {
            format
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_background_color(Color::RGB(CARD))
                .set_border_top(FormatBorder::Medium)
                .set_border_top_color(Color::RGB(NAVY))
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
            kpi_label: card_borders(
                base(8)
                    .set_bold()
                    .set_font_color(Color::RGB(GRAY))
                    .set_background_color(Color::RGB(CARD)),
            )
            .set_border_top(FormatBorder::Medium)
            .set_border_top_color(Color::RGB(BLUE))
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::Bottom)
            .set_indent(1),
            kpi_value: card_borders(
                base(18)
                    .set_bold()
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(CARD)),
            )
            .set_num_format(NUMBER_MONEY)
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_indent(1),
            kpi_value_blue: card_borders(
                base(18)
                    .set_bold()
                    .set_font_color(Color::RGB(BLUE))
                    .set_background_color(Color::RGB(CARD)),
            )
            .set_num_format(NUMBER_MONEY)
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_indent(1),
            kpi_margin: card_borders(
                base(18)
                    .set_bold()
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(CARD)),
            )
            .set_num_format(NUMBER_PERCENT_1)
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_indent(1),
            kpi_net: card_borders(
                base(18)
                    .set_bold()
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(CARD)),
            )
            .set_num_format(NUMBER_MONEY_RED)
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_indent(1),
            kpi_caption: card_borders(
                base(8)
                    .set_font_color(Color::RGB(GRAY))
                    .set_background_color(Color::RGB(CARD)),
            )
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::Top)
            .set_indent(1),
            section: base(11)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            waterfall_header_left: table_header(base(9))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            waterfall_header_right: table_header(base(9))
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            waterfall_label: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            waterfall_money: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            waterfall_percent: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            subtotal_label: base(10)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_background_color(Color::RGB(CARD))
                .set_border_top(FormatBorder::Thin)
                .set_border_top_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            subtotal_money: base(10)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_background_color(Color::RGB(CARD))
                .set_border_top(FormatBorder::Thin)
                .set_border_top_color(Color::RGB(NAVY))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            subtotal_percent: base(10)
                .set_bold()
                .set_font_color(Color::RGB(NAVY))
                .set_background_color(Color::RGB(CARD))
                .set_border_top(FormatBorder::Thin)
                .set_border_top_color(Color::RGB(NAVY))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            gross_label: base(10)
                .set_bold()
                .set_font_color(Color::RGB(BLUE))
                .set_background_color(Color::RGB(GROSS_BG))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            gross_money: base(10)
                .set_bold()
                .set_font_color(Color::RGB(BLUE))
                .set_background_color(Color::RGB(GROSS_BG))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            gross_percent: base(10)
                .set_bold()
                .set_font_color(Color::RGB(BLUE))
                .set_background_color(Color::RGB(GROSS_BG))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            final_label: base(10)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            final_money: base(10)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            final_percent: base(10)
                .set_bold()
                .set_font_color(Color::RGB(WHITE))
                .set_background_color(Color::RGB(NAVY))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            breakdown_label: base(10)
                .set_italic()
                .set_font_color(Color::RGB(GRAY))
                .set_border_bottom(FormatBorder::Dotted)
                .set_border_bottom_color(Color::RGB(BORDER))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(3),
            breakdown_money: base(10)
                .set_italic()
                .set_font_color(Color::RGB(GRAY))
                .set_border_bottom(FormatBorder::Dotted)
                .set_border_bottom_color(Color::RGB(BORDER))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            breakdown_percent: base(10)
                .set_italic()
                .set_font_color(Color::RGB(GRAY))
                .set_border_bottom(FormatBorder::Dotted)
                .set_border_bottom_color(Color::RGB(BORDER))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            notes_heading: base(10).set_bold().set_font_color(Color::RGB(NAVY)),
            note: base(9)
                .set_font_color(Color::RGB(GRAY))
                .set_text_wrap()
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::Top),
            note_italic: base(9)
                .set_italic()
                .set_font_color(Color::RGB(GRAY))
                .set_text_wrap()
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::Top),
            sheet_title: base(14).set_bold().set_font_color(Color::RGB(NAVY)),
            sheet_subtitle: base(9).set_font_color(Color::RGB(GRAY)),
            orders_header_left: table_header(base(9))
                .set_text_wrap()
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            orders_header_center: table_header(base(9))
                .set_text_wrap()
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter),
            orders_header_right: table_header(base(9))
                .set_text_wrap()
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            order_text: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            order_date: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_DATE)
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter),
            order_money: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            order_percent: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            order_text_banded: row_border(
                base(10)
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(BAND)),
            )
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_indent(1),
            order_date_banded: row_border(
                base(10)
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(BAND)),
            )
            .set_num_format(NUMBER_DATE)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter),
            order_money_banded: row_border(
                base(10)
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(BAND)),
            )
            .set_num_format(NUMBER_MONEY)
            .set_align(FormatAlign::Right)
            .set_align(FormatAlign::VerticalCenter),
            order_percent_banded: row_border(
                base(10)
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(BAND)),
            )
            .set_num_format(NUMBER_PERCENT_1)
            .set_align(FormatAlign::Right)
            .set_align(FormatAlign::VerticalCenter),
            orders_total_label: pale_total(base(10))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            orders_total_money: pale_total(base(10))
                .set_num_format(NUMBER_MONEY)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            orders_total_percent: pale_total(base(10))
                .set_num_format(NUMBER_PERCENT_1)
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            orders_note: base(9)
                .set_italic()
                .set_font_color(Color::RGB(GRAY))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter),
            missing_cost: Format::new()
                .set_italic()
                .set_font_color(Color::RGB(WARN_FG))
                .set_background_color(Color::RGB(WARN_BG)),
            expenses_header_center: table_header(base(9))
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter),
            expenses_header_left: table_header(base(9))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            expenses_header_right: table_header(base(9))
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            expense_vat: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_num_format(NUMBER_PERCENT_0)
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter),
            expense_vat_banded: row_border(
                base(10)
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(BAND)),
            )
            .set_num_format(NUMBER_PERCENT_0)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter),
            expenses_total_label: pale_total(base(10))
                .set_align(FormatAlign::Left)
                .set_align(FormatAlign::VerticalCenter)
                .set_indent(1),
            expenses_total_empty: pale_total(base(10))
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            categories_blank: row_border(base(10).set_font_color(Color::RGB(DARK)))
                .set_align(FormatAlign::Right)
                .set_align(FormatAlign::VerticalCenter),
            categories_blank_banded: row_border(
                base(10)
                    .set_font_color(Color::RGB(DARK))
                    .set_background_color(Color::RGB(BAND)),
            )
            .set_align(FormatAlign::Right)
            .set_align(FormatAlign::VerticalCenter),
            service_label: base(10).set_font_color(Color::RGB(GRAY)),
            service_number: base(10)
                .set_font_color(Color::RGB(GRAY))
                .set_num_format(NUMBER_INTEGER),
        }
    }

    /// The label, money and percent formats of a revenue-to-result row.
    fn waterfall_row(&self, kind: WaterfallKind) -> (&Format, &Format, &Format) {
        match kind {
            WaterfallKind::Plain => (
                &self.waterfall_label,
                &self.waterfall_money,
                &self.waterfall_percent,
            ),
            WaterfallKind::Subtotal => (
                &self.subtotal_label,
                &self.subtotal_money,
                &self.subtotal_percent,
            ),
            WaterfallKind::Gross => (&self.gross_label, &self.gross_money, &self.gross_percent),
            WaterfallKind::Breakdown => (
                &self.breakdown_label,
                &self.breakdown_money,
                &self.breakdown_percent,
            ),
            WaterfallKind::Final => (&self.final_label, &self.final_money, &self.final_percent),
        }
    }
}

/// A decimal as the number Excel should store for a money cell.
pub(crate) fn money(value: rust_decimal::Decimal) -> f64 {
    value.round_dp(2).to_f64().unwrap_or_default()
}

/// A ratio as the number Excel should store for a percent cell.
pub(crate) fn fraction(value: rust_decimal::Decimal) -> f64 {
    value.to_f64().unwrap_or_default()
}

/// A calendar day as Excel's 1900-based serial number.
pub(crate) fn excel_day(day: NaiveDate) -> f64 {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30).expect("the Excel epoch is a valid day");
    (day - epoch).num_days() as f64
}

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

fn page_setup(worksheet: &mut Worksheet) {
    worksheet
        .set_landscape()
        .set_paper_size(9)
        .set_margins(0.5, 0.5, 0.6, 0.7, 0.51, 0.5)
        .set_print_center_horizontally(true)
        .set_footer(FOOTER);
}

fn draw_summary(
    worksheet: &mut Worksheet,
    formats: &Formats,
    report: &ReportFigures,
    kind: ReportKind,
    chart: &Chart,
) -> Result<(), XlsxError> {
    let f = formats;
    worksheet.set_name(SHEET_SUMMARY)?;
    worksheet.set_screen_gridlines(false);

    for (col, width) in SUMMARY_WIDTHS.iter().enumerate() {
        worksheet.set_column_width(col as u16, *width)?;
    }
    for (row, height) in SUMMARY_HEIGHTS {
        worksheet.set_row_height(row, height)?;
    }

    worksheet.merge_range(0, 1, 0, 8, kind.title(), &f.banner_title)?;
    worksheet.merge_range(1, 1, 1, 8, &report.subtitle, &f.banner_subtitle)?;

    let labels = [
        "ВИРУЧКА, ГРН",
        "ВАЛОВИЙ ДОХІД, ГРН",
        "МАРЖА",
        "ЧИСТИЙ РЕЗУЛЬТАТ, ГРН",
    ];
    let captions = [
        report.kpi.orders_caption.as_str(),
        "виручка − собівартість",
        "валовий дохід / виручка",
        "після витрат поза замовленнями",
    ];
    let card_columns = [1u16, 3, 5, 7];

    for (index, col) in card_columns.iter().copied().enumerate() {
        worksheet.merge_range(3, col, 3, col + 1, labels[index], &f.kpi_label)?;

        match index {
            0 => merge_number(
                worksheet,
                4,
                col,
                4,
                col + 1,
                money(report.kpi.revenue),
                &f.kpi_value,
            )?,
            1 => merge_number(
                worksheet,
                4,
                col,
                4,
                col + 1,
                money(report.kpi.gross),
                &f.kpi_value_blue,
            )?,
            2 => merge_number(
                worksheet,
                4,
                col,
                4,
                col + 1,
                fraction(report.kpi.margin),
                &f.kpi_margin,
            )?,
            _ => merge_number(
                worksheet,
                4,
                col,
                4,
                col + 1,
                money(report.kpi.net),
                &f.kpi_net,
            )?,
        }

        worksheet.merge_range(5, col, 5, col + 1, captions[index], &f.kpi_caption)?;
    }

    worksheet.merge_range(7, 1, 7, 4, "Від виручки до результату", &f.section)?;
    worksheet.merge_range(7, 5, 7, 8, "Порівняння показників", &f.section)?;

    worksheet.merge_range(8, 1, 8, 2, "Показник", &f.waterfall_header_left)?;
    worksheet.write_string_with_format(8, 3, "Сума, грн", &f.waterfall_header_right)?;
    worksheet.write_string_with_format(8, 4, "% виручки", &f.waterfall_header_right)?;

    for (index, row) in report.waterfall.iter().enumerate() {
        let at = 9 + index as u32;
        let (label, money_format, percent_format) = f.waterfall_row(row.kind);

        worksheet.merge_range(at, 1, at, 2, row.label, label)?;
        worksheet.write_number_with_format(at, 3, money(row.amount), money_format)?;
        worksheet.write_number_with_format(at, 4, fraction(row.percent), percent_format)?;
    }

    worksheet.write_string_with_format(21, 1, "Примітки", &f.notes_heading)?;
    for (index, note) in NOTES.iter().enumerate() {
        let at = 22 + index as u32;
        worksheet.merge_range(at, 1, at, 8, note, &f.note)?;
    }
    worksheet.merge_range(25, 1, 25, 8, NOTE_GENERATED, &f.note_italic)?;

    worksheet.insert_chart_with_offset(8, 5, chart, 0, 0)?;

    worksheet.set_print_fit_to_pages(1, 1);
    worksheet.set_print_area(0, 0, 25, 9)?;
    page_setup(worksheet);

    Ok(())
}

fn draw_orders(
    worksheet: &mut Worksheet,
    formats: &Formats,
    report: &ReportFigures,
) -> Result<(), XlsxError> {
    let f = formats;
    worksheet.set_name(SHEET_ORDERS)?;
    worksheet.set_screen_gridlines(false);

    for (col, width) in ORDERS_WIDTHS.iter().enumerate() {
        worksheet.set_column_width(col as u16, *width)?;
    }

    for (row, height) in [(0, 24.0), (1, 15.0), (2, 7.5), (3, 33.75)] {
        worksheet.set_row_height(row, height)?;
    }

    worksheet.write_string_with_format(0, 0, ORDERS_TITLE, &f.sheet_title)?;
    worksheet.write_string_with_format(1, 0, &report.subtitle, &f.sheet_subtitle)?;

    let headers: [(&str, &Format); 13] = [
        ("№ замовлення", &f.orders_header_left),
        ("Дата оплати", &f.orders_header_center),
        ("Спосіб оплати", &f.orders_header_left),
        ("Клієнт", &f.orders_header_left),
        ("Авто", &f.orders_header_left),
        ("Робота, грн", &f.orders_header_right),
        ("Запчастини, грн", &f.orders_header_right),
        ("Знижка, грн", &f.orders_header_right),
        ("Виручка, грн", &f.orders_header_right),
        ("Собівартість роботи, грн", &f.orders_header_right),
        ("Собівартість запчастин, грн", &f.orders_header_right),
        ("Валовий дохід, грн", &f.orders_header_right),
        ("Маржа", &f.orders_header_right),
    ];
    for (col, (text, format)) in headers.into_iter().enumerate() {
        worksheet.write_string_with_format(3, col as u16, text, format)?;
    }

    for (index, order) in report.orders.iter().enumerate() {
        let row = 4 + index as u32;
        let banded = index % 2 == 1;
        worksheet.set_row_height(row, 19.5)?;

        let text = if banded {
            &f.order_text_banded
        } else {
            &f.order_text
        };
        let date = if banded {
            &f.order_date_banded
        } else {
            &f.order_date
        };
        let money_format = if banded {
            &f.order_money_banded
        } else {
            &f.order_money
        };
        let percent = if banded {
            &f.order_percent_banded
        } else {
            &f.order_percent
        };

        worksheet.write_string_with_format(row, 0, &order.id, text)?;
        worksheet.write_number_with_format(row, 1, excel_day(order.paid_on), date)?;
        worksheet.write_string_with_format(row, 2, &order.payment, text)?;
        worksheet.write_string_with_format(row, 3, &order.client, text)?;
        worksheet.write_string_with_format(row, 4, &order.car, text)?;
        worksheet.write_number_with_format(row, 5, money(order.labor), money_format)?;
        worksheet.write_number_with_format(row, 6, money(order.parts), money_format)?;
        worksheet.write_number_with_format(row, 7, money(order.discount), money_format)?;
        worksheet.write_number_with_format(row, 8, money(order.revenue), money_format)?;

        match order.labor_cost {
            Some(cost) => {
                worksheet.write_number_with_format(row, 9, money(cost), money_format)?;
            }
            None => {
                worksheet.write_string_with_format(row, 9, "не вказано", money_format)?;
                let note = Note::new(NOTE_MISSING_LABOR_COST).set_author(NOTE_AUTHOR);
                worksheet.insert_note(row, 9, &note)?;
            }
        }

        match order.parts_cost {
            Some(cost) => {
                worksheet.write_number_with_format(row, 10, money(cost), money_format)?;
            }
            None => {
                worksheet.write_string_with_format(row, 10, "не вказано", money_format)?;
                let note = Note::new(NOTE_MISSING_PARTS_COST).set_author(NOTE_AUTHOR);
                worksheet.insert_note(row, 10, &note)?;
            }
        }

        worksheet.write_number_with_format(row, 11, money(order.gross), money_format)?;
        worksheet.write_number_with_format(row, 12, fraction(order.margin), percent)?;
    }

    let totals_row = 4 + report.orders.len() as u32;
    worksheet.set_row_height(totals_row, 24.0)?;
    worksheet.write_string_with_format(totals_row, 0, "Разом", &f.orders_total_label)?;
    for col in 1..=4 {
        worksheet.write_blank(totals_row, col, &f.orders_total_money)?;
    }

    let totals = [
        (5, report.totals.labor),
        (6, report.totals.parts),
        (7, report.totals.discount),
        (8, report.totals.revenue),
        (9, report.totals.labor_cost),
        (10, report.totals.parts_cost),
        (11, report.totals.gross),
    ];
    for (col, value) in totals {
        worksheet.write_number_with_format(totals_row, col, money(value), &f.orders_total_money)?;
    }
    worksheet.write_number_with_format(
        totals_row,
        12,
        fraction(report.totals.margin),
        &f.orders_total_percent,
    )?;

    let note_row = totals_row + 2;
    worksheet.set_row_height(note_row, 15.0)?;
    worksheet.write_string_with_format(note_row, 0, ORDERS_NOTE, &f.orders_note)?;

    let last_data_row = 3 + report.orders.len() as u32;
    if !report.orders.is_empty() {
        worksheet.autofilter(3, 0, last_data_row, 12)?;

        let missing_cost = ConditionalFormatFormula::new()
            .set_rule("OR(NOT(ISNUMBER($J5)),NOT(ISNUMBER($K5)))")
            .set_format(f.missing_cost.clone());
        worksheet.add_conditional_format(4, 9, last_data_row, 12, &missing_cost)?;

        let revenue_bar = ConditionalFormatDataBar::new()
            .set_minimum(ConditionalFormatType::Number, 0)
            .set_maximum(ConditionalFormatType::Highest, 0)
            .set_fill_color(Color::RGB(DATA_BAR));
        worksheet.add_conditional_format(4, 8, last_data_row, 8, &revenue_bar)?;
    }

    worksheet.set_print_fit_to_pages(1, 0);
    worksheet.set_print_area(0, 0, note_row, 12)?;
    worksheet.set_repeat_rows(3, 3)?;
    page_setup(worksheet);
    worksheet.set_freeze_panes(4, 2)?;

    Ok(())
}

fn draw_expenses(
    worksheet: &mut Worksheet,
    formats: &Formats,
    report: &ReportFigures,
) -> Result<(), XlsxError> {
    let f = formats;
    worksheet.set_name(SHEET_EXPENSES)?;
    worksheet.set_screen_gridlines(false);

    for (col, width) in EXPENSES_WIDTHS.iter().enumerate() {
        worksheet.set_column_width(col as u16, *width)?;
    }
    for (row, height) in [(0, 24.0), (1, 15.0), (2, 7.5), (3, 25.5)] {
        worksheet.set_row_height(row, height)?;
    }

    worksheet.write_string_with_format(0, 0, EXPENSES_TITLE, &f.sheet_title)?;
    worksheet.write_string_with_format(1, 0, &report.subtitle, &f.sheet_subtitle)?;

    let headers: [(&str, &Format); 6] = [
        ("Дата", &f.expenses_header_center),
        ("Опис", &f.expenses_header_left),
        ("Категорія", &f.expenses_header_left),
        ("Сума, грн", &f.expenses_header_right),
        ("ПДВ", &f.expenses_header_center),
        ("Спосіб оплати", &f.expenses_header_left),
    ];
    for (col, (text, format)) in headers.into_iter().enumerate() {
        worksheet.write_string_with_format(3, col as u16, text, format)?;
    }

    for (index, expense) in report.expenses.iter().enumerate() {
        let row = 4 + index as u32;
        let banded = index % 2 == 1;
        worksheet.set_row_height(row, 19.5)?;

        let text = if banded {
            &f.order_text_banded
        } else {
            &f.order_text
        };
        let date = if banded {
            &f.order_date_banded
        } else {
            &f.order_date
        };
        let money_format = if banded {
            &f.order_money_banded
        } else {
            &f.order_money
        };
        let vat = if banded {
            &f.expense_vat_banded
        } else {
            &f.expense_vat
        };

        match expense.date {
            Some(day) => worksheet.write_number_with_format(row, 0, excel_day(day), date)?,
            None => worksheet.write_string_with_format(row, 0, &expense.date_text, date)?,
        };
        worksheet.write_string_with_format(row, 1, &expense.description, text)?;
        worksheet.write_string_with_format(row, 2, &expense.category, text)?;
        worksheet.write_number_with_format(row, 3, money(expense.amount), money_format)?;
        worksheet.write_number_with_format(
            row,
            4,
            fraction(expense.vat_rate / rust_decimal::Decimal::ONE_HUNDRED),
            vat,
        )?;
        worksheet.write_string_with_format(row, 5, &expense.payment, text)?;
    }

    let totals_row = 4 + report.expenses.len() as u32;
    worksheet.set_row_height(totals_row, 24.0)?;
    worksheet.write_string_with_format(totals_row, 0, "Разом", &f.expenses_total_label)?;
    for col in [1u16, 2, 4, 5] {
        worksheet.write_blank(totals_row, col, &f.expenses_total_empty)?;
    }
    worksheet.write_number_with_format(
        totals_row,
        3,
        money(report.totals.expenses),
        &f.orders_total_money,
    )?;

    let mut last_row = totals_row;

    if !report.categories.is_empty() {
        let heading_row = totals_row + 2;
        worksheet.set_row_height(heading_row, 21.75)?;
        worksheet.write_string_with_format(heading_row, 0, "За категоріями", &f.section)?;

        let header_row = heading_row + 1;
        worksheet.set_row_height(header_row, 21.75)?;
        worksheet.write_blank(header_row, 0, &f.expenses_header_right)?;
        worksheet.write_string_with_format(header_row, 1, "Категорія", &f.expenses_header_left)?;
        worksheet.write_blank(header_row, 2, &f.expenses_header_right)?;
        worksheet.write_string_with_format(header_row, 3, "Сума, грн", &f.expenses_header_right)?;
        worksheet.write_string_with_format(header_row, 4, "Частка", &f.expenses_header_right)?;
        worksheet.write_blank(header_row, 5, &f.expenses_header_right)?;

        for (index, category) in report.categories.iter().enumerate() {
            let row = header_row + 1 + index as u32;
            let banded = index % 2 == 1;
            worksheet.set_row_height(row, 19.5)?;

            let text = if banded {
                &f.order_text_banded
            } else {
                &f.order_text
            };
            let money_format = if banded {
                &f.order_money_banded
            } else {
                &f.order_money
            };
            let percent = if banded {
                &f.order_percent_banded
            } else {
                &f.order_percent
            };
            let blank = if banded {
                &f.categories_blank_banded
            } else {
                &f.categories_blank
            };

            worksheet.write_blank(row, 0, blank)?;
            worksheet.write_string_with_format(row, 1, &category.name, text)?;
            worksheet.write_blank(row, 2, blank)?;
            worksheet.write_number_with_format(row, 3, money(category.amount), money_format)?;
            worksheet.write_number_with_format(row, 4, fraction(category.share), percent)?;
            worksheet.write_blank(row, 5, blank)?;
            last_row = row;
        }

        let category_total_row = last_row + 1;
        worksheet.set_row_height(category_total_row, 24.0)?;
        worksheet.write_blank(category_total_row, 0, &f.expenses_total_empty)?;
        worksheet.write_string_with_format(
            category_total_row,
            1,
            "Разом",
            &f.expenses_total_label,
        )?;
        worksheet.write_blank(category_total_row, 2, &f.expenses_total_empty)?;
        worksheet.write_number_with_format(
            category_total_row,
            3,
            money(report.totals.expenses),
            &f.orders_total_money,
        )?;
        worksheet.write_number_with_format(
            category_total_row,
            4,
            if report.totals.expenses.is_zero() {
                0.0
            } else {
                1.0
            },
            &f.orders_total_percent,
        )?;
        worksheet.write_blank(category_total_row, 5, &f.expenses_total_empty)?;
        last_row = category_total_row;

        let share_bar = ConditionalFormatDataBar::new()
            .set_minimum(ConditionalFormatType::Number, 0)
            .set_maximum(ConditionalFormatType::Number, 1)
            .set_fill_color(Color::RGB(DATA_BAR));
        worksheet.add_conditional_format(
            header_row + 1,
            4,
            header_row + report.categories.len() as u32,
            4,
            &share_bar,
        )?;
    }

    if !report.expenses.is_empty() {
        worksheet.autofilter(3, 0, 3 + report.expenses.len() as u32, 5)?;
    }

    worksheet.set_print_fit_to_pages(1, 0);
    worksheet.set_print_area(0, 0, last_row, 5)?;
    worksheet.set_repeat_rows(3, 3)?;
    page_setup(worksheet);
    worksheet.set_freeze_panes(4, 0)?;

    Ok(())
}

fn draw_service(
    worksheet: &mut Worksheet,
    formats: &Formats,
    report: &ReportFigures,
) -> Result<(), XlsxError> {
    let f = formats;
    worksheet.set_name(SHEET_SERVICE)?;
    worksheet.set_hidden(true);
    worksheet.set_screen_gridlines(false);

    for (col, width) in SERVICE_WIDTHS.iter().enumerate() {
        worksheet.set_column_width(col as u16, *width)?;
    }
    worksheet.set_row_height(0, 17.35)?;
    for row in [1u32, 3, 4, 5, 6, 8, 9, 10, 11] {
        worksheet.set_row_height(row, 15.0)?;
    }

    worksheet.write_string_with_format(0, 0, "Службові дані", &f.sheet_title)?;
    worksheet.write_string_with_format(
        1,
        0,
        "Допоміжний лист: джерело даних для графіка та автоматичні перевірки. Не редагуйте вручну.",
        &f.sheet_subtitle,
    )?;

    worksheet.write_string_with_format(3, 0, "Дані для графіка", &f.section)?;
    let chart_labels = ["Виручка", "Валовий дохід", "Чистий результат"];
    for (index, label) in chart_labels.iter().enumerate() {
        let row = 4 + index as u32;
        worksheet.write_string_with_format(row, 0, *label, &f.service_label)?;
        worksheet.write_number_with_format(
            row,
            1,
            money(report.chart[index]),
            &f.service_number,
        )?;
    }

    worksheet.write_string_with_format(8, 0, "Перевірки", &f.section)?;
    worksheet.write_string_with_format(
        9,
        0,
        "Замовлень без собівартості робіт або запчастин",
        &f.service_label,
    )?;
    worksheet.write_number_with_format(
        9,
        1,
        report.checks.missing_cost as f64,
        &f.service_number,
    )?;
    worksheet.write_string_with_format(
        10,
        0,
        "Валовий дохід за напрямами = валовий дохід",
        &f.service_label,
    )?;
    worksheet.write_string_with_format(
        10,
        1,
        check_label(report.checks.gross_consistent),
        &f.service_label,
    )?;
    worksheet.write_string_with_format(
        11,
        0,
        "Виручка за напрямами = виручка",
        &f.service_label,
    )?;
    worksheet.write_string_with_format(
        11,
        1,
        check_label(report.checks.revenue_consistent),
        &f.service_label,
    )?;

    Ok(())
}

fn check_label(consistent: bool) -> &'static str {
    if consistent {
        "OK"
    } else {
        "Розбіжність"
    }
}

fn build_chart(report: &ReportFigures) -> Chart {
    let mut chart = Chart::new(ChartType::Column);
    chart.set_width(400);
    chart.set_height(264);

    chart
        .add_series()
        .set_categories((SHEET_SERVICE, 4, 0, 6, 0))
        .set_values((SHEET_SERVICE, 4, 1, 6, 1))
        .set_point_colors(&[Color::RGB(CHART_LIGHT), Color::RGB(BLUE), Color::RGB(NAVY)])
        .set_data_label(
            ChartDataLabel::new()
                .show_value()
                .set_position(ChartDataLabelPosition::OutsideEnd)
                .set_num_format(NUMBER_INTEGER),
        )
        .set_gap(60);

    let mut axis_line = ChartLine::new();
    axis_line.set_color(Color::RGB(CHART_AXIS));
    let mut label_font = ChartFont::new();
    label_font.set_size(10);

    chart
        .x_axis()
        .set_format(ChartFormat::new().set_line(&axis_line))
        .set_font(&label_font);
    chart
        .y_axis()
        .set_format(ChartFormat::new().set_line(&axis_line))
        .set_font(&label_font)
        .set_num_format(NUMBER_INTEGER);

    chart.y_axis().set_major_gridlines(true);
    chart
        .y_axis()
        .set_major_gridlines_line(ChartLine::new().set_color(Color::RGB(BORDER)));

    // With nothing to compare the automatic scale would run 0 to 1 and label
    // both ends as "1"; one unit between 0 and 1 keeps the empty chart legible.
    if report.chart.iter().all(|value| value.is_zero()) {
        chart.y_axis().set_min(0.0).set_max(1.0).set_major_unit(1.0);
    }

    chart.legend().set_hidden();

    chart.chart_area().set_format(
        ChartFormat::new()
            .set_solid_fill(ChartSolidFill::new().set_color(Color::RGB(WHITE)))
            .set_border(ChartLine::new().set_color(Color::RGB(CHART_BORDER))),
    );
    chart
        .plot_area()
        .set_format(ChartFormat::new().set_no_fill().set_no_line());

    chart
}

fn build_workbook(kind: ReportKind, report: &ReportFigures) -> Result<Workbook, XlsxError> {
    let formats = Formats::new();
    let chart = build_chart(report);

    let mut workbook = Workbook::new();
    let properties = DocProperties::new()
        .set_title(format!(
            "{} {} – {}",
            kind.title(),
            report.period.start.format("%d.%m.%Y"),
            report.period.end.format("%d.%m.%Y")
        ))
        .set_subject("Фінансовий звіт за період")
        .set_company(&report.company)
        .set_author(&report.company);
    workbook.set_properties(&properties);

    {
        let worksheet = workbook.add_worksheet();
        draw_summary(worksheet, &formats, report, kind, &chart)?;
    }
    {
        let worksheet = workbook.add_worksheet();
        draw_orders(worksheet, &formats, report)?;
    }
    {
        let worksheet = workbook.add_worksheet();
        draw_expenses(worksheet, &formats, report)?;
    }
    {
        let worksheet = workbook.add_worksheet();
        draw_service(worksheet, &formats, report)?;
    }

    Ok(workbook)
}

/// The workbook of one report, in memory.
fn render_figures(kind: ReportKind, report: &ReportFigures) -> Result<Vec<u8>, XlsxError> {
    build_workbook(kind, report)?.save_to_buffer()
}

pub async fn render(
    state: &State<'_, AppState>,
    kind: ReportKind,
    range: DayRange,
    sources: &ReportSources,
) -> AppResult<Vec<u8>> {
    let settings = crate::database::settings::fetch_settings_from_pool(&state.db_pool).await?;
    let context = ReportContext::from_settings(&settings, Utc::now());
    let figures = ReportFigures::new(&context, range, sources);

    render_figures(kind, &figures)
        .map_err(|error| AppError::Generic(format!("Не вдалося сформувати XLSX: {error}")))
}
