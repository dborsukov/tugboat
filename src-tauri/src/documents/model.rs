use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use num2words::{Currency, Lang, Num2Words};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        clients::structs::{Client, ClientDetails, ClientType},
        settings::structs::{AppSettings, CompanyInfo, CompanyKind},
        work_orders::{
            dtos::{VatBucket, WorkOrderDetailsDto},
            pricing::{self, calculate_work_order_totals},
            structs::{DiscountType, LaborPriceType},
        },
        workers::structs::Worker,
    },
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    NaryadZamovlennya,
    AktVykonanykhRobit,
}

impl DocumentKind {
    pub fn title(self) -> &'static str {
        match self {
            DocumentKind::NaryadZamovlennya => "Наряд-замовлення",
            DocumentKind::AktVykonanykhRobit => "Акт виконаних робіт",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    Pdf,
    Xlsx,
}

impl DocumentFormat {
    pub fn extension(self) -> &'static str {
        match self {
            DocumentFormat::Pdf => "pdf",
            DocumentFormat::Xlsx => "xlsx",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentConflictAction {
    Replace,
    Rename,
}

/// Extra information collected from the user right before generation.
#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GenerateDocumentOptions {
    pub performer_worker_id: Option<i32>,
    pub received_at: Option<DateTime<Utc>>,
    pub planned_completion_at: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub odometer_out: Option<i32>,
    pub additional_recommendations: Option<String>,
    #[serde(default)]
    pub skip_date: bool,
    #[serde(default)]
    pub skip_location: bool,
    #[serde(default)]
    pub skip_performer_signature: bool,
    #[serde(default)]
    pub skip_customer_signature: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderDocument {
    pub name: String,
    pub path: String,
    #[specta(type = specta_typescript::Number)]
    pub size: i64,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderDocuments {
    pub folder_path: String,
    pub files: Vec<WorkOrderDocument>,
}

// ---------------------------------------------------------------------------
// Internal document model shared by all renderers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DocClient {
    pub id: i64,
    pub client_type: ClientType,
    pub name: String,
    pub surname: Option<String>,
    pub first_name: Option<String>,
    pub patronymic: Option<String>,
    pub phone: String,
    pub legal_name: Option<String>,
    pub legal_address: Option<String>,
    pub edrpou: Option<String>,
    pub rnokpp: Option<String>,
    pub director_pib: Option<String>,
    pub iban: Option<String>,
    pub bank_name: Option<String>,
}

impl DocClient {
    pub fn from_parts(client: &Client, details: Option<&ClientDetails>) -> Self {
        let phone = details
            .and_then(|d| trim(&d.contact_phone))
            .or_else(|| trim(&client.phone))
            .unwrap_or_default();
        let director_pib = details.and_then(|d| {
            let pib = join_pib(
                &d.director_surname,
                &d.director_first_name,
                &d.director_patronymic,
            );
            if pib.is_empty() {
                None
            } else {
                Some(pib)
            }
        });
        Self {
            id: client.id,
            client_type: client.client_type,
            name: client.name.clone(),
            surname: trim(&client.surname),
            first_name: trim(&client.first_name),
            patronymic: trim(&client.patronymic),
            phone,
            legal_name: details.and_then(|d| trim(&d.legal_name)),
            legal_address: details.and_then(|d| trim(&d.legal_address)),
            edrpou: details.and_then(|d| trim(&d.edrpou)),
            rnokpp: details.and_then(|d| trim(&d.rnokpp)),
            director_pib,
            iban: details.and_then(|d| trim(&d.iban)),
            bank_name: details.and_then(|d| trim(&d.bank_name)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentData {
    pub kind: DocumentKind,
    pub title: String,
    pub subtitle: String,
    pub company: Option<CompanyInfo>,
    pub work_order_id: String,
    pub order_created_at: String,
    pub owner: DocClient,
    pub customer: Option<DocClient>,
    pub payer: Option<DocClient>,
    pub driver_surname: Option<String>,
    pub driver_first_name: Option<String>,
    pub driver_patronymic: Option<String>,
    pub driver_phone: Option<String>,
    pub vat_payer: bool,
    pub car_make_model: String,
    pub car_year: i64,
    pub car_license_plate: String,
    pub car_vin: String,
    pub odometer_in: i32,
    pub condition_report: Option<String>,
    pub description: Option<String>,
    pub doc_date: Option<String>,
    pub doc_location: Option<String>,
    pub performer_signature: String,
    pub customer_signature: String,
    pub received_at: Option<String>,
    pub planned_completion_at: Option<String>,
    pub completed_at: Option<String>,
    pub additional_recommendations: Option<String>,
    pub labor: DocTable,
    pub parts: DocTable,
    pub totals: DocTotals,
    pub total_in_words: String,
}

impl DocumentData {
    pub fn driver_name(&self) -> Option<String> {
        let name = join_pib(
            &self.driver_surname,
            &self.driver_first_name,
            &self.driver_patronymic,
        );
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocTable {
    pub title: String,
    pub rows: Vec<DocTableRow>,
    pub total: Decimal,
}

#[derive(Debug, Clone)]
pub struct DocTableRow {
    pub code: String,
    pub name: String,
    pub quantity: Decimal,
    pub unit: String,
    pub price: Decimal,
    pub discount_text: String,
    pub discounted_price: Decimal,
    pub sum: Decimal,
}

#[derive(Debug, Clone)]
pub struct DocTotals {
    pub labor_total: Decimal,
    pub parts_total: Decimal,
    pub subtotal: Decimal,
    pub order_discount: Decimal,
    pub order_discount_is_percentage: bool,
    pub order_discount_percent: Decimal,
    pub grand_total: Decimal,
    pub vat_buckets: Vec<VatBucket>,
    pub total_with_vat: Decimal,
    pub show_vat: bool,
    pub prices_include_vat: bool,
}

/// The label a VAT row carries, e.g. `20 %` or `0 %, експорт`.
pub fn vat_bucket_label(bucket: &VatBucket) -> String {
    let rate = format!("{} %", format_qty(bucket.rate));
    if bucket.scheme.is_standard() {
        rate
    } else {
        format!("{rate}, {}", bucket.scheme.label().to_lowercase())
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// `1234.5` -> `"1 234,50"`
pub fn format_money(value: Decimal) -> String {
    let rounded = value.round_dp(2);
    let neg = rounded.is_sign_negative();
    let s = rounded.abs().to_string();
    let (int_part, frac_part) = match s.split_once('.') {
        Some((int, frac)) => (int.to_string(), frac.chars().take(2).collect::<String>()),
        None => (s, String::new()),
    };
    let frac_padded = format!("{frac_part:0<2}");

    let mut grouped = String::new();
    let chars: Vec<char> = int_part.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        if idx > 0 && (chars.len() - idx).is_multiple_of(3) {
            grouped.push(' ');
        }
        grouped.push(*ch);
    }

    let sign = if neg { "-" } else { "" };
    format!("{sign}{grouped},{frac_padded}")
}

/// Trimmed decimal with Ukrainian comma: `2.50` -> `"2,5"`, `1` -> `"1"`.
pub fn format_qty(value: Decimal) -> String {
    value.normalize().to_string().replace('.', ",")
}

fn format_datetime(dt: DateTime<Utc>, tz: Tz) -> String {
    dt.with_timezone(&tz).format("%d.%m.%Y %H:%M").to_string()
}

fn format_date(dt: DateTime<Utc>, tz: Tz) -> String {
    dt.with_timezone(&tz).format("%d.%m.%Y").to_string()
}

fn trim(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn join_pib(
    surname: &Option<String>,
    first_name: &Option<String>,
    patronymic: &Option<String>,
) -> String {
    [surname, first_name, patronymic]
        .iter()
        .filter_map(|v| trim(v))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn pib_initials(surname: &str, first_name: &str, patronymic: &str) -> String {
    let surname = surname.trim();
    if surname.is_empty() {
        return [first_name, patronymic]
            .iter()
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
    }

    let initials = [first_name, patronymic]
        .iter()
        .filter_map(|part| part.trim().chars().next().map(|c| format!("{c}.")))
        .collect::<Vec<_>>()
        .join("");

    if initials.is_empty() {
        surname.to_string()
    } else {
        format!("{surname} {initials}")
    }
}

pub fn legal_display_name(client: &DocClient) -> String {
    match client.client_type {
        ClientType::Individual => client.name.clone(),
        ClientType::Fop => client
            .legal_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("ФОП {}", client.name)),
        ClientType::Company => client
            .legal_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| client.name.clone()),
    }
}

pub fn client_tax_code_line(client: &DocClient) -> Option<String> {
    match client.client_type {
        ClientType::Individual => None,
        ClientType::Fop => client.rnokpp.clone().map(|v| format!("РНОКПП: {v}")),
        ClientType::Company => client.edrpou.clone().map(|v| format!("ЄДРПОУ: {v}")),
    }
}

pub fn client_bank_info_lines(client: &DocClient) -> Vec<String> {
    bank_lines(client.iban.as_deref(), client.bank_name.as_deref())
}

pub fn customer_signature_name(
    client: &DocClient,
    driver_surname: &Option<String>,
    driver_first_name: &Option<String>,
    driver_patronymic: &Option<String>,
) -> String {
    match client.client_type {
        ClientType::Individual => {
            let initials = pib_initials(
                client.surname.as_deref().unwrap_or(""),
                client.first_name.as_deref().unwrap_or(""),
                client.patronymic.as_deref().unwrap_or(""),
            );
            if initials.trim().is_empty() {
                client.name.clone()
            } else {
                initials
            }
        }
        ClientType::Fop | ClientType::Company => {
            let driver = join_pib(driver_surname, driver_first_name, driver_patronymic);
            if driver.is_empty() {
                String::new()
            } else {
                driver
            }
        }
    }
}

pub fn performer_signature_name(
    position: &Option<String>,
    surname: &str,
    first_name: &str,
    patronymic: &str,
) -> String {
    let pib = pib_initials(surname, first_name, patronymic);
    match trim(position) {
        Some(position) if !pib.trim().is_empty() => format!("{position} {pib}"),
        Some(position) => position,
        None => pib,
    }
}

pub fn total_in_words(total: Decimal) -> String {
    let rounded = total.round_dp(2);
    let value = rounded.to_f64().unwrap_or(0.0);
    let words = Num2Words::new(value)
        .lang(Lang::Ukrainian)
        .currency(Currency::UAH)
        .to_words()
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format_money(rounded));
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => words,
    }
}

// ---------------------------------------------------------------------------
// Model construction
// ---------------------------------------------------------------------------

fn discount_text(discount_type: DiscountType, value: Decimal) -> String {
    match discount_type {
        DiscountType::None => String::new(),
        DiscountType::Fixed => format!("-{}", format_money(value)),
        DiscountType::Percentage => format!("{} %", format_qty(value)),
    }
}

pub fn company_name_line(info: &CompanyInfo) -> String {
    if let Some(name) = trim(&info.name) {
        return name;
    }
    company_full_legal_line(info)
}

pub fn company_full_legal_line(info: &CompanyInfo) -> String {
    match info.kind {
        CompanyKind::Fop => {
            let pib = join_pib(&info.surname, &info.first_name, &info.patronymic);
            if pib.is_empty() {
                info.name.clone().unwrap_or_default()
            } else {
                format!("ФОП {pib}")
            }
        }
        CompanyKind::Company => info
            .legal_name
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| info.name.clone())
            .unwrap_or_default(),
    }
}

pub fn company_code_parts(info: &CompanyInfo) -> Option<(&'static str, String)> {
    match info.kind {
        CompanyKind::Fop => info.rnokpp.clone().map(|v| ("РНОКПП", v)),
        CompanyKind::Company => info.edrpou.clone().map(|v| ("ЄДРПОУ", v)),
    }
}

pub fn company_bank_lines(info: &CompanyInfo) -> Vec<String> {
    bank_lines(info.iban.as_deref(), info.bank_name.as_deref())
}

fn bank_lines(iban: Option<&str>, bank: Option<&str>) -> Vec<String> {
    let iban = iban.map(str::trim).filter(|s| !s.is_empty());
    let bank = bank.map(str::trim).filter(|s| !s.is_empty());
    match (iban, bank) {
        (Some(iban), Some(bank)) => vec![format!("Р/Р: {iban}"), format!("Банк (МФО): {bank}")],
        (Some(iban), None) => vec![format!("Р/Р: {iban}")],
        (None, Some(bank)) => vec![format!("Банк (МФО): {bank}")],
        (None, None) => Vec::new(),
    }
}

fn build_labor_table(details: &WorkOrderDetailsDto) -> DocTable {
    let mut rows = Vec::new();
    for dto in &details.labor_items {
        if !pricing::labor_is_billed(dto.item.is_free) {
            continue;
        }
        let (quantity, unit, price) = match dto.item.price_type {
            LaborPriceType::Hourly => (dto.item.labor_hours, "год.", dto.totals.price.round_dp(2)),
            LaborPriceType::Fixed => (Decimal::ONE, "штука", dto.totals.price.round_dp(2)),
        };
        let discounted_price = dto.totals.discounted_price.round_dp(2);
        rows.push(DocTableRow {
            code: dto.item.code.clone().unwrap_or_default(),
            name: dto.item.name.clone(),
            quantity,
            unit: unit.to_string(),
            price,
            discount_text: discount_text(dto.item.discount_type, dto.item.discount_value),
            discounted_price,
            sum: dto.totals.total,
        });
    }

    let total: Decimal = rows.iter().map(|r| r.sum).sum();

    DocTable {
        title: "Виконані роботи".to_string(),
        rows,
        total,
    }
}

fn build_parts_table(details: &WorkOrderDetailsDto, kind: DocumentKind) -> DocTable {
    let mut rows = Vec::new();
    for dto in &details.part_items {
        let billed = match kind {
            DocumentKind::NaryadZamovlennya => {
                pricing::part_is_projected_billed(dto.item.status, dto.item.is_free)
            }
            DocumentKind::AktVykonanykhRobit => {
                pricing::part_is_billed(dto.item.status, dto.item.is_free)
            }
        };
        if !billed {
            continue;
        }
        rows.push(DocTableRow {
            code: dto.item.code.clone().unwrap_or_default(),
            name: dto.item.name.clone(),
            quantity: dto.item.quantity,
            unit: dto.item.unit_type.clone(),
            price: dto.totals.retail_price,
            discount_text: discount_text(dto.item.discount_type, dto.item.discount_value),
            discounted_price: dto.totals.discounted_price,
            sum: dto.totals.total,
        });
    }

    let total: Decimal = rows.iter().map(|r| r.sum).sum();

    DocTable {
        title: "Запчастини та витратні матеріали".to_string(),
        rows,
        total,
    }
}

pub fn build_document_data(
    details: &WorkOrderDetailsDto,
    settings: &AppSettings,
    kind: DocumentKind,
    options: &GenerateDocumentOptions,
    customer: Option<DocClient>,
    payer: Option<DocClient>,
    performer: Option<&Worker>,
) -> AppResult<DocumentData> {
    let work_order = &details.work_order;
    let tz: Tz = settings
        .timezone
        .parse()
        .unwrap_or(chrono_tz::Tz::Europe__Kiev);

    let vat_payer = settings.company_info.as_ref().is_some_and(|c| c.vat_payer);

    let labor = build_labor_table(details);
    let parts = build_parts_table(details, kind);

    // The lines the VAT is allocated over, in the same set the printed tables show.
    let lines: Vec<pricing::PricedLine> = details
        .labor_items
        .iter()
        .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
        .map(|dto| {
            let (rate, scheme) = pricing::line_vat(
                dto.item.vat_rate,
                dto.item.vat_scheme,
                work_order.vat_rate,
                work_order.vat_scheme,
            );
            pricing::PricedLine {
                kind: pricing::LineKind::Labor,
                total: dto.totals.total,
                retail: dto.totals.total + dto.totals.discount,
                rate,
                scheme,
            }
        })
        .chain(
            details
                .part_items
                .iter()
                .filter(|dto| match kind {
                    DocumentKind::NaryadZamovlennya => {
                        pricing::part_is_projected_billed(dto.item.status, dto.item.is_free)
                    }
                    DocumentKind::AktVykonanykhRobit => {
                        pricing::part_is_billed(dto.item.status, dto.item.is_free)
                    }
                })
                .map(|dto| {
                    let (rate, scheme) = pricing::line_vat(
                        dto.item.vat_rate,
                        dto.item.vat_scheme,
                        work_order.vat_rate,
                        work_order.vat_scheme,
                    );
                    pricing::PricedLine {
                        kind: pricing::LineKind::Parts,
                        total: dto.totals.total,
                        retail: dto.totals.total + dto.totals.discount,
                        rate,
                        scheme,
                    }
                }),
        )
        .collect();

    let order_totals = calculate_work_order_totals(
        labor.total,
        Decimal::ZERO,
        parts.total,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        work_order.discount_type,
        work_order.discount_value,
        work_order.prices_include_vat,
        &lines,
    );

    let show_vat = vat_payer && !order_totals.vat_buckets.is_empty();

    let totals = DocTotals {
        labor_total: labor.total,
        parts_total: parts.total,
        subtotal: order_totals.subtotal,
        order_discount: order_totals.order_discount,
        order_discount_is_percentage: work_order.discount_type == DiscountType::Percentage,
        order_discount_percent: work_order.discount_value,
        grand_total: order_totals.grand_total,
        vat_buckets: order_totals.vat_buckets.clone(),
        total_with_vat: order_totals.total_with_vat,
        show_vat,
        prices_include_vat: work_order.prices_include_vat,
    };

    let owner = DocClient::from_parts(&details.client, details.client_details.as_ref());
    let effective_customer = customer.clone().unwrap_or_else(|| owner.clone());

    let performer_signature = if options.skip_performer_signature {
        String::new()
    } else {
        let performer = performer.ok_or_else(|| AppError::validation("Оберіть виконавця"))?;
        if performer.surname.trim().is_empty() || performer.first_name.trim().is_empty() {
            return Err(AppError::validation(
                "У виконавця не заповнено ПІБ. Відредагуйте працівника в довіднику.",
            ));
        }
        performer_signature_name(
            &performer.position,
            &performer.surname,
            &performer.first_name,
            performer.patronymic.as_deref().unwrap_or(""),
        )
    };
    let customer_signature = if options.skip_customer_signature {
        String::new()
    } else {
        customer_signature_name(
            &effective_customer,
            &work_order.driver_surname,
            &work_order.driver_first_name,
            &work_order.driver_patronymic,
        )
    };

    let driver_surname = trim(&work_order.driver_surname);
    let driver_first_name = trim(&work_order.driver_first_name);
    let driver_patronymic = trim(&work_order.driver_patronymic);
    let driver_phone = trim(&work_order.driver_phone);

    let total_in_words = total_in_words(totals.total_with_vat);

    let base = DocumentData {
        kind,
        title: String::new(),
        subtitle: String::new(),
        company: settings.company_info.clone(),
        work_order_id: work_order.id.clone(),
        order_created_at: format_date(work_order.created_at, tz),
        owner,
        customer,
        payer,
        driver_surname,
        driver_first_name,
        driver_patronymic,
        driver_phone,
        vat_payer,
        car_make_model: format!(
            "{} {}",
            details.car_dto.brand.name, details.car_dto.car.model
        ),
        car_year: details.car_dto.car.year,
        car_license_plate: details
            .car_dto
            .car
            .license_plate
            .clone()
            .unwrap_or_default(),
        car_vin: details.car_dto.car.vin.clone().unwrap_or_default(),
        odometer_in: work_order.odometer_in,
        condition_report: trim(&work_order.condition_report),
        description: trim(&work_order.description),
        doc_date: None,
        doc_location: None,
        performer_signature,
        customer_signature,
        received_at: None,
        planned_completion_at: None,
        completed_at: None,
        additional_recommendations: trim(&options.additional_recommendations),
        labor,
        parts,
        totals,
        total_in_words,
    };

    let data = match kind {
        DocumentKind::NaryadZamovlennya => {
            let received_at = if options.skip_date {
                None
            } else {
                Some(options.received_at.ok_or_else(|| {
                    AppError::validation("Вкажіть дату та час прийняття автомобіля")
                })?)
            };
            let planned_completion_at = options.planned_completion_at.ok_or_else(|| {
                AppError::validation("Вкажіть орієнтовний термін виконання робіт")
            })?;

            DocumentData {
                title: format!("НАРЯД-ЗАМОВЛЕННЯ № {}", work_order.id),
                subtitle: match &received_at {
                    Some(dt) => format!("від {}", format_date(*dt, tz)),
                    None => "від ________________".to_string(),
                },
                doc_date: received_at.map(|dt| format_date(dt, tz)),
                received_at: received_at.map(|dt| format_datetime(dt, tz)),
                planned_completion_at: Some(format_datetime(planned_completion_at, tz)),
                ..base
            }
        }
        DocumentKind::AktVykonanykhRobit => {
            let location = if options.skip_location {
                None
            } else {
                Some(
                    trim(&options.location)
                        .ok_or_else(|| AppError::validation("Вкажіть місце складання акту"))?,
                )
            };
            let completed_at = if options.skip_date {
                None
            } else {
                Some(
                    options
                        .completed_at
                        .ok_or_else(|| AppError::validation("Вкажіть дату складання акту"))?,
                )
            };
            options.odometer_out.ok_or_else(|| {
                AppError::validation("Вкажіть пробіг на момент видачі автомобіля")
            })?;

            let subtitle_parts: Vec<String> = [
                location.as_ref().cloned(),
                completed_at.map(|dt| format_date(dt, tz)),
            ]
            .into_iter()
            .flatten()
            .collect();

            DocumentData {
                title: format!("АКТ ВИКОНАНИХ РОБІТ № {}", work_order.id),
                subtitle: subtitle_parts.join(", "),
                doc_date: completed_at.map(|dt| format_date(dt, tz)),
                doc_location: location.clone(),
                completed_at: completed_at.map(|dt| format_date(dt, tz)),
                ..base
            }
        }
    };

    Ok(data)
}
