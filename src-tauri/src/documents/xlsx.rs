use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};

use super::model::{
    client_tax_code_line, company_code_parts, company_name_line, DocumentData, DocumentKind,
};
use crate::{
    database::clients::structs::ClientType,
    error::{AppError, AppResult},
};

fn trimmed(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// "Виконавець" (performer) block for the table header.
fn company_header_lines(data: &DocumentData) -> Vec<(String, String)> {
    let mut lines = Vec::new();
    match &data.company {
        None => {
            lines.push((
                "Виконавець".to_string(),
                "Дані бізнесу не заповнено".to_string(),
            ));
        }
        Some(info) => {
            lines.push(("Виконавець".to_string(), company_name_line(info)));
            if let Some((label, value)) = company_code_parts(info) {
                lines.push((label.to_string(), value));
            }
            if let Some(addr) = trimmed(&info.legal_address) {
                lines.push(("Адреса".to_string(), addr));
            }
            if let Some(phone) = trimmed(&info.phone) {
                lines.push(("Телефон".to_string(), phone));
            }
        }
    }
    lines
}

fn client_info_lines(data: &DocumentData) -> Vec<(String, String)> {
    let owner = &data.owner;
    let mut lines = vec![("Замовник".to_string(), owner.name.clone())];
    if let Some(code) = client_tax_code_line(owner) {
        let legal = super::model::legal_display_name(owner);
        lines.push(("Реквізити".to_string(), format!("{legal}, {code}")));
    } else if owner.client_type != ClientType::Individual {
        lines.push((
            "Реквізити".to_string(),
            super::model::legal_display_name(owner),
        ));
    }
    if !owner.phone.is_empty() {
        lines.push(("Телефон".to_string(), owner.phone.clone()));
    }
    lines
}

fn car_info_lines(data: &DocumentData) -> Vec<(String, String)> {
    let mut lines = vec![("Автомобіль".to_string(), data.car_make_model.clone())];
    lines.push(("Рік випуску".to_string(), data.car_year.to_string()));
    if !data.car_license_plate.is_empty() {
        lines.push((
            "Державний номер".to_string(),
            data.car_license_plate.clone(),
        ));
    }
    if !data.car_vin.is_empty() {
        lines.push(("VIN".to_string(), data.car_vin.clone()));
    }
    lines.push((
        "Пробіг при прийнятті".to_string(),
        format!("{} км", data.odometer_in),
    ));
    lines
}

/// The "Документ-підстава" (basis document) line for the acceptance act.
fn act_basis_line(data: &DocumentData) -> String {
    format!(
        "наряд-замовлення № {} від {}",
        data.work_order_id, data.order_created_at
    )
}

const TABLE_HEADERS: [&str; 8] = [
    "№",
    "Код",
    "Найменування",
    "Кількість",
    "Ціна",
    "Знижка",
    "Відпускна ціна",
    "Сума",
];

fn money_f64(v: rust_decimal::Decimal) -> f64 {
    v.round_dp(2).to_string().parse().unwrap_or(0.0)
}

fn write_item_table(
    worksheet: &mut Worksheet,
    start_row: &mut u32,
    table: &super::model::DocTable,
    header: &Format,
    cell_text: &Format,
    cell_number: &Format,
    money: &Format,
) -> Result<(), AppError> {
    // Title row (merged)
    worksheet.merge_range(*start_row, 0, *start_row, 7, &table.title, header)?;
    *start_row += 1;

    // Header row
    for (col, h) in TABLE_HEADERS.iter().enumerate() {
        worksheet.write_string_with_format(*start_row, col as u16, *h, header)?;
    }
    *start_row += 1;

    for (i, row) in table.rows.iter().enumerate() {
        let r = *start_row;
        worksheet.write_number_with_format(r, 0, (i + 1) as f64, cell_number)?;
        worksheet.write_string_with_format(r, 1, &row.code, cell_text)?;
        worksheet.write_string_with_format(r, 2, &row.name, cell_text)?;
        worksheet.write_string_with_format(r, 3, &row.quantity_text, cell_number)?;
        worksheet.write_number_with_format(r, 4, money_f64(row.price), money)?;
        worksheet.write_string_with_format(r, 5, &row.discount_text, cell_text)?;
        worksheet.write_number_with_format(r, 6, money_f64(row.discounted_price), money)?;
        worksheet.write_number_with_format(r, 7, money_f64(row.sum), money)?;
        *start_row += 1;
    }

    // Total
    worksheet.merge_range(*start_row, 0, *start_row, 6, "Всього", header)?;
    worksheet.write_number_with_format(*start_row, 7, money_f64(table.total), header)?;
    *start_row += 1;

    Ok(())
}

fn write_customer_parts_table(
    worksheet: &mut Worksheet,
    start_row: &mut u32,
    table: &super::model::DocTable,
    header: &Format,
    cell_text: &Format,
    cell_number: &Format,
) -> Result<(), AppError> {
    worksheet.merge_range(*start_row, 0, *start_row, 7, &table.title, header)?;
    *start_row += 1;

    for (col, h) in TABLE_HEADERS.iter().enumerate() {
        worksheet.write_string_with_format(*start_row, col as u16, *h, header)?;
    }
    *start_row += 1;

    let muted_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_font_color(Color::RGB(0x5B6772));

    for (i, row) in table.rows.iter().enumerate() {
        let r = *start_row;
        worksheet.write_number_with_format(r, 0, (i + 1) as f64, cell_number)?;
        worksheet.write_string_with_format(r, 1, &row.code, cell_text)?;
        worksheet.write_string_with_format(r, 2, &row.name, cell_text)?;
        worksheet.write_string_with_format(r, 3, &row.quantity_text, cell_number)?;
        worksheet.merge_range(r, 4, r, 7, "—", &muted_format)?;
        *start_row += 1;
    }

    Ok(())
}

pub fn render(data: &DocumentData) -> AppResult<Vec<u8>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center);
    let subtitle_format = Format::new()
        .set_font_size(10)
        .set_align(FormatAlign::Center);
    let label_format = Format::new().set_bold();
    let section_format = Format::new().set_bold().set_border(FormatBorder::Thin);
    let header_format = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let cell_text_format = Format::new().set_border(FormatBorder::Thin);
    let cell_number_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let money_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_num_format("#,##0.00")
        .set_align(FormatAlign::Right);

    worksheet.set_column_width(0, 4.0)?;
    worksheet.set_column_width(1, 12.0)?;
    worksheet.set_column_width(2, 48.0)?;
    worksheet.set_column_width(3, 14.0)?;
    worksheet.set_column_width(4, 12.0)?;
    worksheet.set_column_width(5, 10.0)?;
    worksheet.set_column_width(6, 14.0)?;
    worksheet.set_column_width(7, 12.0)?;

    let mut row: u32 = 0;

    worksheet.merge_range(row, 0, row, 7, &data.title, &title_format)?;
    row += 1;
    worksheet.merge_range(row, 0, row, 7, &data.subtitle, &subtitle_format)?;
    row += 2;

    // Business info
    for (label, value) in company_header_lines(data) {
        worksheet.write_string_with_format(row, 0, label, &label_format)?;
        worksheet.write_string_with_format(row, 1, value, &cell_text_format)?;
        row += 1;
    }
    row += 1;

    // Client info
    for (label, value) in client_info_lines(data) {
        worksheet.write_string_with_format(row, 0, label, &label_format)?;
        worksheet.write_string_with_format(row, 1, value, &cell_text_format)?;
        row += 1;
    }
    row += 1;

    // Car info
    for (label, value) in car_info_lines(data) {
        worksheet.write_string_with_format(row, 0, label, &label_format)?;
        worksheet.write_string_with_format(row, 1, value, &cell_text_format)?;
        row += 1;
    }
    row += 1;

    match data.kind {
        DocumentKind::NaryadZamovlennya => {
            let extra = [
                (
                    "Дата та час прийняття",
                    data.received_at.clone().unwrap_or_default(),
                ),
                (
                    "Орієнтовний термін виконання",
                    data.planned_completion_at.clone().unwrap_or_default(),
                ),
                ("Виконавець", data.performer_signature.clone()),
                (
                    "Зняті/старі запчастини необхідно (підкресліть одне)",
                    "повернути замовнику / залишити виконавцю".to_string(),
                ),
            ];
            for (label, value) in extra {
                worksheet.write_string_with_format(row, 0, label, &label_format)?;
                worksheet.write_string_with_format(row, 1, value, &cell_text_format)?;
                row += 1;
            }
        }
        DocumentKind::AktVykonanykhRobit => {
            let extra = [
                ("Документ-підстава", act_basis_line(data)),
                (
                    "Місце та дата складання",
                    [
                        data.doc_location.as_deref().unwrap_or("—"),
                        data.doc_date.as_deref().unwrap_or("—"),
                    ]
                    .join(", "),
                ),
            ];
            for (label, value) in extra {
                worksheet.write_string_with_format(row, 0, label, &label_format)?;
                worksheet.write_string_with_format(row, 1, value, &cell_text_format)?;
                row += 1;
            }
        }
    }
    row += 1;

    // Item tables
    write_item_table(
        worksheet,
        &mut row,
        &data.labor,
        &header_format,
        &cell_text_format,
        &cell_number_format,
        &money_format,
    )?;
    row += 1;
    write_item_table(
        worksheet,
        &mut row,
        &data.parts,
        &header_format,
        &cell_text_format,
        &cell_number_format,
        &money_format,
    )?;
    row += 1;

    if !data.customer_parts.rows.is_empty() {
        write_customer_parts_table(
            worksheet,
            &mut row,
            &data.customer_parts,
            &header_format,
            &cell_text_format,
            &cell_number_format,
        )?;
        row += 1;
    }

    // Grand totals
    let t = &data.totals;
    let mut totals: Vec<(String, f64)> = vec![
        ("Роботи, всього".to_string(), money_f64(t.labor_total)),
        (
            "Запчастини та матеріали, всього".to_string(),
            money_f64(t.parts_total),
        ),
        ("Разом".to_string(), money_f64(t.subtotal)),
    ];
    if t.order_discount > rust_decimal::Decimal::ZERO {
        // For a percentage discount show the percentage itself — as for VAT.
        let discount_label = if t.order_discount_is_percentage {
            format!(
                "Знижка за замовленням ({} %)",
                super::model::format_qty(t.order_discount_percent)
            )
        } else {
            "Знижка за замовленням".to_string()
        };
        totals.push((discount_label, -money_f64(t.order_discount)));
        totals.push(("Всього зі знижкою".to_string(), money_f64(t.grand_total)));
    }
    if t.show_vat {
        for bucket in &t.vat_buckets {
            let label = super::model::vat_bucket_label(bucket);
            if t.prices_include_vat {
                totals.push((format!("Без ПДВ ({label})"), money_f64(bucket.base)));
            }
            totals.push((format!("ПДВ ({label})"), money_f64(bucket.amount)));
        }
    }
    totals.push(("До сплати".to_string(), money_f64(t.total_with_vat)));

    worksheet.write_string_with_format(row, 0, "Підсумок", &section_format)?;
    row += 1;
    for (label, value) in totals {
        worksheet.write_string_with_format(row, 0, label, &label_format)?;
        worksheet.write_number_with_format(row, 7, value, &money_format)?;
        row += 1;
    }

    let buffer: Vec<u8> = workbook
        .save_to_buffer()
        .map_err(|e| AppError::Generic(format!("Не вдалося створити XLSX: {e}")))?;
    Ok(buffer)
}
