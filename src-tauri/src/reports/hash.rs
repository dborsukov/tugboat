//! Source hashing for report staleness check.
//! The hash is built from individual records, never from aggregated totals.

use blake3::Hasher;

use super::structs::DayRange;
use crate::{
    database::{
        clients::structs::ClientType,
        expenses::structs::Expense,
        vat::structs::VatEntry,
        work_orders::structs::{LaborItem, PartItem, WorkOrder},
    },
    hashing::FieldHash,
};

/// Domain separator per record kind.
/// Avoids collisions between identical items from different domains.
const DOMAIN_WORK_ORDER: &[u8] = b"work_order";
const DOMAIN_LABOR_ITEM: &[u8] = b"labor_item";
const DOMAIN_PART_ITEM: &[u8] = b"part_item";
const DOMAIN_EXPENSE: &[u8] = b"expense";
const DOMAIN_ORDER_PARTY: &[u8] = b"order_party";
const DOMAIN_VAT_ENTRY: &[u8] = b"vat_entry";

const VAT_REPORT_FORMAT: &[u8] = b"vat-register-v4";

#[derive(Debug, Clone)]
pub struct OrderParty {
    pub order_id: String,
    pub client_id: i64,
    pub client_name: String,
    pub client_surname: Option<String>,
    pub client_first_name: Option<String>,
    pub client_patronymic: Option<String>,
    pub client_type: ClientType,
    pub car_id: i64,
    pub car_brand: Option<String>,
    pub car_model: String,
}

fn hash_work_order(hasher: &mut Hasher, order: &WorkOrder) {
    hasher.field(DOMAIN_WORK_ORDER);
    hasher.text(&order.id);
    hasher.number(order.car_id);
    hasher.optional_number(order.payer_client_id.map(i64::from));
    hasher.optional_number(order.customer_client_id.map(i64::from));
    hasher.optional_number(order.worker_id.map(i64::from));
    hasher.number(i64::from(order.odometer_in));
    hasher.optional_number(order.odometer_out.map(i64::from));
    hasher.optional_text(order.driver_surname.as_deref());
    hasher.optional_text(order.driver_first_name.as_deref());
    hasher.optional_text(order.driver_patronymic.as_deref());
    hasher.optional_text(order.driver_phone.as_deref());
    hasher.optional_text(order.description.as_deref());
    hasher.optional_text(order.notes.as_deref());
    hasher.optional_text(order.condition_report.as_deref());
    hasher.text(&format!("{:?}", order.status));
    hasher.text(&format!("{:?}", order.r#type));
    hasher.optional_text(order.warranty_of.as_deref());
    hasher.decimal(order.rate);
    hasher.text(&format!("{:?}", order.labor_cost_type));
    hasher.decimal(order.labor_cost_value);
    hasher.text(&format!("{:?}", order.discount_type));
    hasher.decimal(order.discount_value);
    hasher.decimal(order.vat_rate);
    hasher.flag(order.prices_include_vat);
    hasher.timestamp(order.created_at);
    hasher.optional_timestamp(order.archived_at);
    hasher.optional_timestamp(order.started_at);
    hasher.optional_timestamp(order.completed_at);
    hasher.optional_timestamp(order.paid_at);
    hasher.optional_text(
        order
            .payment_method
            .map(|method| format!("{method:?}"))
            .as_deref(),
    );
}

fn hash_labor_item(hasher: &mut Hasher, item: &LaborItem) {
    hasher.field(DOMAIN_LABOR_ITEM);
    hasher.number(item.id);
    hasher.text(&item.work_order_id);
    hasher.optional_number(item.service_id);
    hasher.optional_text(item.code.as_deref());
    hasher.text(&item.name);
    hasher.text(&format!("{:?}", item.price_type));
    hasher.decimal(item.fixed_price);
    hasher.decimal(item.labor_hours);
    hasher.text(&format!("{:?}", item.discount_type));
    hasher.decimal(item.discount_value);
    hasher.flag(item.is_free);
    hasher.number(i64::from(item.position));
}

fn hash_part_item(hasher: &mut Hasher, item: &PartItem) {
    hasher.field(DOMAIN_PART_ITEM);
    hasher.number(item.id);
    hasher.text(&item.work_order_id);
    hasher.text(&format!("{:?}", item.status));
    hasher.optional_text(item.code.as_deref());
    hasher.text(&item.name);
    hasher.decimal(item.quantity);
    hasher.optional_text(item.unit_type.as_deref());
    hasher.optional_text(item.unit_key.as_deref());
    hasher.decimal(item.base_cost);
    hasher.decimal(item.markup);
    hasher.text(&format!("{:?}", item.discount_type));
    hasher.decimal(item.discount_value);
    hasher.flag(item.is_free);
    hasher.text(&format!("{:?}", item.provided_by));
    hasher.number(i64::from(item.position));
}

fn hash_expense(hasher: &mut Hasher, expense: &Expense) {
    hasher.field(DOMAIN_EXPENSE);
    hasher.number(expense.id);
    hasher.text(&expense.date);
    hasher.decimal(expense.amount);
    hasher.text(&expense.description);
    // The category id, not its name, renaming a category should not trigger stale state.
    hasher.optional_number(expense.category.as_ref().map(|category| category.id));
    hasher.optional_text(
        expense
            .payment_method
            .map(|method| format!("{method:?}"))
            .as_deref(),
    );
    hasher.decimal(expense.vat_rate);
    hasher.flag(expense.price_includes_vat);
    hasher.timestamp(expense.created_at);
    hasher.optional_timestamp(expense.archived_at);
}

fn hash_order_party(hasher: &mut Hasher, party: &OrderParty) {
    hasher.field(DOMAIN_ORDER_PARTY);
    hasher.text(&party.order_id);
    hasher.number(party.client_id);
    // The printed name is built from these parts, so a rename is a change.
    hasher.text(&party.client_name);
    hasher.optional_text(party.client_surname.as_deref());
    hasher.optional_text(party.client_first_name.as_deref());
    hasher.optional_text(party.client_patronymic.as_deref());
    hasher.text(&format!("{:?}", party.client_type));
    hasher.number(party.car_id);
    hasher.optional_text(party.car_brand.as_deref());
    hasher.text(&party.car_model);
}

/// One fixed VAT row, including the reversal link and the void it may carry.
fn hash_vat_entry(hasher: &mut Hasher, entry: &VatEntry) {
    hasher.field(DOMAIN_VAT_ENTRY);
    hasher.number(entry.id);
    hasher.text(&format!("{:?}", entry.direction));
    hasher.text(&entry.tax_point);
    hasher.text(&format!("{:?}", entry.source_kind));
    hasher.text(&entry.source_id);
    hasher.optional_text(entry.document_number.as_deref());
    hasher.optional_text(entry.document_date.as_deref());
    hasher.text(&entry.counterparty_name);
    hasher.optional_text(entry.counterparty_code.as_deref());
    hasher.flag(entry.counterparty_vat_payer);
    hasher.decimal(entry.base);
    hasher.decimal(entry.rate);
    hasher.decimal(entry.amount);
    hasher.text(&format!("{:?}", entry.scheme));
    hasher.text(&format!("{:?}", entry.exemption_code));
    hasher.optional_number(entry.reverses_id);
    hasher.optional_timestamp(entry.voided_at);
    hasher.timestamp(entry.created_at);
}

/// The records a VAT report of a period is built from.
pub fn vat_entries_hash(entries: &[VatEntry]) -> String {
    let mut hasher = Hasher::new();
    hasher.field(VAT_REPORT_FORMAT);

    let mut entries: Vec<&VatEntry> = entries.iter().collect();
    entries.sort_by_key(|entry| entry.id);
    for entry in entries {
        hash_vat_entry(&mut hasher, entry);
    }

    hasher.finalize().to_hex().to_string()
}

/// Everything a report of `range` is built from.
pub struct ReportSources {
    pub orders: Vec<WorkOrder>,
    pub labor_items: Vec<LaborItem>,
    pub part_items: Vec<PartItem>,
    pub expenses: Vec<Expense>,
    pub parties: Vec<OrderParty>,
}

impl ReportSources {
    /// Records in a fixed order, so the hash depends on the data alone.
    /// The rows arrive from SQL already ordered, but sorting here as well
    /// protects from query plan changes in the future triggering stale check.
    pub fn hash(&self) -> String {
        let mut hasher = Hasher::new();

        let mut orders: Vec<&WorkOrder> = self.orders.iter().collect();
        orders.sort_by(|a, b| a.id.cmp(&b.id));
        for order in orders {
            hash_work_order(&mut hasher, order);
        }

        let mut labor_items: Vec<&LaborItem> = self.labor_items.iter().collect();
        labor_items.sort_by_key(|item| item.id);
        for item in labor_items {
            hash_labor_item(&mut hasher, item);
        }

        let mut part_items: Vec<&PartItem> = self.part_items.iter().collect();
        part_items.sort_by_key(|item| item.id);
        for item in part_items {
            hash_part_item(&mut hasher, item);
        }

        let mut expenses: Vec<&Expense> = self.expenses.iter().collect();
        expenses.sort_by_key(|expense| expense.id);
        for expense in expenses {
            hash_expense(&mut hasher, expense);
        }

        let mut parties: Vec<&OrderParty> = self.parties.iter().collect();
        parties.sort_by(|a, b| a.order_id.cmp(&b.order_id));
        for party in parties {
            hash_order_party(&mut hasher, party);
        }

        hasher.finalize().to_hex().to_string()
    }
}

#[allow(dead_code)]
fn _range_is_used(range: DayRange) -> String {
    range.folder_name()
}
