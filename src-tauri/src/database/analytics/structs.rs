use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::database::expenses::structs::ExpenseCategoryColor;

/// A half open time window: `from` is inclusive, `to` is exclusive
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// Every window a dashboard module may need, resolved once by the frontend in the configured timezone
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsPeriod {
    pub month: DateRange,
    pub previous_month: DateRange,
    pub week: DateRange,
    pub previous_week: DateRange,
}

/// A month the shop was paid in, in the shop's own timezone.
///
/// Months are what the dashboard can be pointed at, and a month is worth pointing at
/// when money changed hands in it: an order taken in one month and settled in another
/// belongs to the month it was settled in, the way every money tile counts it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OrderMonth {
    pub year: i32,
    pub month: i32,
    /// Orders paid in that month.
    pub order_count: i32,
}

/// Finished work that has not been paid for.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OutstandingBalance {
    pub total: Decimal,
    pub order_count: i32,
}

/// Work still under way, valued at what it is expected to bill.
///
/// The counterpart of [`OutstandingBalance`]: that one is money waiting to come in
/// for work that is finished, this is money tied up in work that is not. The two
/// count different orders, so no order is in both.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InProgressBalance {
    pub total: Decimal,
    pub order_count: i32,
}

/// Output VAT recorded in the ledger this month, by tax point.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VatDue {
    pub amount: Decimal,
    /// Supplies with an entry this month
    pub order_count: i32,
}

/// What the orders of one period earned, net of VAT.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RevenueBucket {
    pub total: Decimal,
    pub order_count: i32,
}

/// What a month's revenue was earned on.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RevenueSplit {
    pub labor: Decimal,
    pub parts: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RevenueSummary {
    pub month: RevenueBucket,
    pub previous_month: RevenueBucket,
    pub week: RevenueBucket,
    pub previous_week: RevenueBucket,
    /// Running revenue of the month, net of VAT: what was earned through the end of each
    /// day, first day first, so the last entry is `month.total`. A day the shop took no
    /// money on repeats the day before, and the days after the last one are left out.
    pub month_by_day: Vec<Decimal>,
    pub previous_month_by_day: Vec<Decimal>,
    /// Revenue split by labor and parts.
    pub month_split: RevenueSplit,
}

/// One service of the period, as a top-services tile draws it.
/// A service is the name on a line: the same name billed twice in the month is one row.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TopServiceRow {
    pub name: String,
    /// Times it was billed in the period.
    pub count: i32,
    /// What it earned the shop over the period, net of VAT.
    pub revenue: Decimal,
    /// That revenue per time it was billed.
    pub average_price: Decimal,
    /// How many more or fewer times it was billed than the month before.
    pub change: Option<i32>,
}

/// The period's services, and the two wholes their shares can be shares of.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TopServicesReport {
    pub total_revenue: Decimal,
    pub services_revenue: Decimal,
    pub services: Vec<TopServiceRow>,
}

/// New versus returning clients over a period.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientMix {
    pub new_clients: i32,
    pub returning_clients: i32,
    pub total_clients: i32,
    pub previous_total_clients: i32,
}

/// One client of the period, as a tile lists them.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientMixEntry {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    /// Name as it reads in a list
    pub name: String,
    /// The car they came in first, make and model.
    pub car: String,
    /// Number of additional cars they brought this month.
    pub other_cars: i32,
    /// Orders the client placed in the period.
    pub visits: i32,
    /// What those orders earned, total client spending this month, net of VAT.
    pub spent: Decimal,
    /// Whether the period holds their very first order.
    pub is_new: bool,
}

/// The clients behind a period's mix, biggest spender first.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientMixList {
    pub average_check: Decimal,
    pub clients: Vec<ClientMixEntry>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TopServicesOptions {
    pub limit: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MarginOptions {
    pub include_labor_cost: bool,
}

/// Gross margin and gross income over a period, excluding VAT.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    /// Billed revenue excluding VAT.
    pub revenue: Decimal,
    /// Cost of the billed items: parts always, labor unless disabled by options.
    pub cost: Decimal,
    pub income: Decimal,
    #[specta(type = specta_typescript::Number)]
    pub margin_percent: f64,
    /// Previous period, so the tile can tell "no data" from a real 0%.
    pub previous_revenue: Decimal,
    pub previous_income: Decimal,
    #[specta(type = specta_typescript::Number)]
    pub previous_margin_percent: f64,
    pub order_count: i32,
    pub period_order_count: i32,
}

/// What one service contributed to the period's gross income.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MarginService {
    pub name: String,
    pub revenue: Decimal,
    pub cost: Decimal,
    pub income: Decimal,
    #[specta(type = specta_typescript::Number)]
    pub margin_percent: f64,
}

/// A closed range of calendar days, both ends included, as `YYYY-MM-DD` text.
/// Expenses are booked on a day rather than at an instant, so a month of them is a
/// span of days. Orders are counted by the clock instead, which is why the analytics
/// commands take an [`AnalyticsPeriod`] and only the expenses take this.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DaySpan {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseSummary {
    /// Everything booked in the month, net of VAT.
    pub total: Decimal,
    /// The month before.
    pub previous_total: Decimal,
    /// The categories of the month, biggest first.
    pub categories: Vec<ExpenseCategorySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategorySummary {
    pub name: String,
    pub color: ExpenseCategoryColor,
    /// Everything the month booked in this category, net of VAT.
    pub amount: Decimal,
    /// How many expenses the amount is made of.
    pub count: i32,
    /// Share of the month's expenses, in percent.
    #[specta(type = specta_typescript::Number)]
    pub share: f64,
    /// The newest expense of the category.
    pub latest_expense: ExpenseEntrySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseEntrySummary {
    /// The day it was booked on.
    pub date: String,
    pub description: String,
    pub amount: Decimal,
}
