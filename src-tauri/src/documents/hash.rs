//! Source hash for work order documents staleness check.
//! The hash covers the whole [`DocumentData`] the renderers print, not the records it was built from.

use blake3::Hasher;

use super::model::{DocClient, DocTable, DocTotals, DocumentData};
use crate::{database::settings::structs::CompanyInfo, hashing::FieldHash};

const DOCUMENT_FORMAT: &[u8] = b"work-order-document-v1";

const DOMAIN_DOCUMENT: &[u8] = b"document";

fn hash_company(hasher: &mut Hasher, info: &CompanyInfo) {
    hasher.text(&format!("{:?}", info.kind));
    hasher.optional_text(info.name.as_deref());
    hasher.optional_text(info.legal_name.as_deref());
    hasher.optional_text(info.legal_address.as_deref());
    hasher.optional_text(info.physical_address.as_deref());
    hasher.optional_text(info.phone.as_deref());
    hasher.optional_text(info.edrpou.as_deref());
    hasher.optional_text(info.rnokpp.as_deref());
    hasher.optional_text(info.surname.as_deref());
    hasher.optional_text(info.first_name.as_deref());
    hasher.optional_text(info.patronymic.as_deref());
    hasher.optional_text(info.director_surname.as_deref());
    hasher.optional_text(info.director_first_name.as_deref());
    hasher.optional_text(info.director_patronymic.as_deref());
    hasher.optional_text(info.iban.as_deref());
    hasher.optional_text(info.bank_name.as_deref());
    hasher.flag(info.vat_payer);
}

fn hash_client(hasher: &mut Hasher, client: &DocClient) {
    hasher.number(client.id);
    hasher.text(&format!("{:?}", client.client_type));
    hasher.text(&client.name);
    hasher.optional_text(client.surname.as_deref());
    hasher.optional_text(client.first_name.as_deref());
    hasher.optional_text(client.patronymic.as_deref());
    hasher.text(&client.phone);
    hasher.optional_text(client.legal_name.as_deref());
    hasher.optional_text(client.legal_address.as_deref());
    hasher.optional_text(client.edrpou.as_deref());
    hasher.optional_text(client.rnokpp.as_deref());
    hasher.optional_text(client.director_pib.as_deref());
    hasher.optional_text(client.iban.as_deref());
    hasher.optional_text(client.bank_name.as_deref());
}

fn optional_client(hasher: &mut Hasher, client: Option<&DocClient>) {
    match client {
        Some(client) => {
            hasher.field(b"1");
            hash_client(hasher, client);
        }
        None => {
            hasher.field(b"0");
        }
    }
}

fn hash_table(hasher: &mut Hasher, table: &DocTable) {
    hasher.text(&table.title);
    hasher.number(table.rows.len() as i64);
    for row in &table.rows {
        hasher.text(&row.code);
        hasher.text(&row.name);
        hasher.decimal(row.quantity);
        hasher.text(&row.unit);
        hasher.decimal(row.price);
        hasher.text(&row.discount_text);
        hasher.decimal(row.discounted_price);
        hasher.decimal(row.sum);
    }
    hasher.decimal(table.total);
}

fn hash_totals(hasher: &mut Hasher, totals: &DocTotals) {
    hasher.decimal(totals.labor_total);
    hasher.decimal(totals.parts_total);
    hasher.decimal(totals.subtotal);
    hasher.decimal(totals.order_discount);
    hasher.flag(totals.order_discount_is_percentage);
    hasher.decimal(totals.order_discount_percent);
    hasher.decimal(totals.grand_total);
    hasher.decimal(totals.total_with_vat);
    hasher.flag(totals.show_vat);
    hasher.flag(totals.prices_include_vat);
    hasher.number(totals.vat_buckets.len() as i64);
    for bucket in &totals.vat_buckets {
        hasher.decimal(bucket.rate);
        hasher.text(&format!("{:?}", bucket.scheme));
        hasher.decimal(bucket.base);
        hasher.decimal(bucket.amount);
    }
}

/// Everything the printed files are made of, in a fixed order.
pub fn document_hash(data: &DocumentData) -> String {
    let mut hasher = Hasher::new();
    hasher.field(DOCUMENT_FORMAT);
    hasher.field(DOMAIN_DOCUMENT);

    hasher.text(&format!("{:?}", data.kind));
    hasher.text(&data.title);
    hasher.text(&data.subtitle);

    match &data.company {
        Some(info) => {
            hasher.flag(true);
            hash_company(&mut hasher, info);
        }
        None => {
            hasher.flag(false);
        }
    }

    hasher.text(&data.work_order_id);
    hasher.text(&data.order_created_at);
    hash_client(&mut hasher, &data.owner);
    optional_client(&mut hasher, data.customer.as_ref());
    optional_client(&mut hasher, data.payer.as_ref());

    hasher.optional_text(data.driver_surname.as_deref());
    hasher.optional_text(data.driver_first_name.as_deref());
    hasher.optional_text(data.driver_patronymic.as_deref());
    hasher.optional_text(data.driver_phone.as_deref());

    hasher.flag(data.vat_payer);
    hasher.text(&data.car_make_model);
    hasher.number(data.car_year);
    hasher.text(&data.car_license_plate);
    hasher.text(&data.car_vin);
    hasher.number(i64::from(data.odometer_in));

    hasher.optional_text(data.condition_report.as_deref());
    hasher.optional_text(data.description.as_deref());
    hasher.optional_text(data.doc_date.as_deref());
    hasher.optional_text(data.doc_location.as_deref());
    hasher.text(&data.performer_signature);
    hasher.text(&data.customer_signature);
    hasher.optional_text(data.received_at.as_deref());
    hasher.optional_text(data.planned_completion_at.as_deref());
    hasher.optional_text(data.completed_at.as_deref());
    hasher.optional_text(data.additional_recommendations.as_deref());

    hash_table(&mut hasher, &data.labor);
    hash_table(&mut hasher, &data.parts);
    hash_totals(&mut hasher, &data.totals);
    hasher.text(&data.total_in_words);

    hasher.finalize().to_hex().to_string()
}
