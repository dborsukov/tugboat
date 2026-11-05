//! The figures a financial report prints.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use rust_decimal::Decimal;

use super::{
    hash::{OrderParty, ReportSources},
    structs::DayRange,
};
use crate::{
    database::{
        cars::structs::car_display_name,
        clients::structs::client_display_name,
        settings::structs::{AppSettings, CompanyInfo, CompanyKind},
        work_orders::{
            dtos::{LaborItemDto, PartItemDto, WorkOrderTotals},
            pricing::{self, labor_cost_basis_known, labor_is_billed, part_is_billed},
            structs::{LaborItem, PartItem, PaymentMethod, WorkOrder},
        },
    },
    utils::plural,
};

pub(crate) struct ReportContext {
    pub company: String,
    pub timezone: Tz,
    pub generated_at: DateTime<Utc>,
}

impl ReportContext {
    pub fn from_settings(settings: &AppSettings, generated_at: DateTime<Utc>) -> Self {
        Self {
            company: company_name(settings),
            // The same fallback the documents take when the stored zone no
            // longer parses.
            timezone: settings
                .timezone
                .parse()
                .unwrap_or(chrono_tz::Tz::Europe__Kiev),
            generated_at,
        }
    }
}

/// One row of the revenue-to-result table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WaterfallKind {
    Plain,
    Subtotal,
    Gross,
    Breakdown,
    Final,
}

#[derive(Debug, Clone)]
pub(crate) struct WaterfallRow {
    pub label: &'static str,
    pub amount: Decimal,
    pub percent: Decimal,
    pub kind: WaterfallKind,
}

/// One order as the orders sheet prints it.
#[derive(Debug, Clone)]
pub(crate) struct OrderFigure {
    pub id: String,
    pub paid_on: NaiveDate,
    /// How the order was paid, or a dash when the order does not say.
    pub payment: String,
    pub client: String,
    pub car: String,
    /// Work and parts at the price they were billed before any discount.
    pub labor: Decimal,
    pub parts: Decimal,
    /// Everything taken off the two lines above.
    pub discount: Decimal,
    /// Net revenue for the order, net of VAT.
    pub revenue: Decimal,
    /// What the work cost, or `None` when the order tracks hourly costs but
    /// has flat-fee work, which has no hours to measure.
    pub labor_cost: Option<Decimal>,
    /// The purchase cost of the parts, or `None` when it is not known.
    pub parts_cost: Option<Decimal>,
    pub gross: Decimal,
    pub margin: Decimal,
}

/// One expense as the expenses sheet prints it.
#[derive(Debug, Clone)]
pub(crate) struct ExpenseFigure {
    /// `None` when the stored text is not a date; the raw text is printed then.
    pub date: Option<NaiveDate>,
    pub date_text: String,
    pub description: String,
    pub category: String,
    pub amount: Decimal,
    pub vat_rate: Decimal,
    pub payment: String,
}

/// One expense category in the category table.
#[derive(Debug, Clone)]
pub(crate) struct CategoryFigure {
    pub name: String,
    pub amount: Decimal,
    pub share: Decimal,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Totals {
    pub revenue: Decimal,
    pub labor: Decimal,
    pub parts: Decimal,
    pub discount: Decimal,
    pub labor_cost: Decimal,
    pub parts_cost: Decimal,
    pub gross: Decimal,
    pub expenses: Decimal,
    pub net: Decimal,
    pub margin: Decimal,
    pub order_count: usize,
    /// Orders whose labor or parts cost is not fully known, counted once.
    pub missing_cost_count: usize,
}

impl Default for Totals {
    fn default() -> Self {
        Self {
            revenue: Decimal::ZERO,
            labor: Decimal::ZERO,
            parts: Decimal::ZERO,
            discount: Decimal::ZERO,
            labor_cost: Decimal::ZERO,
            parts_cost: Decimal::ZERO,
            gross: Decimal::ZERO,
            expenses: Decimal::ZERO,
            net: Decimal::ZERO,
            margin: Decimal::ZERO,
            order_count: 0,
            missing_cost_count: 0,
        }
    }
}

/// The four headline numbers and the line under them.
#[derive(Debug, Clone)]
pub(crate) struct Kpi {
    pub revenue: Decimal,
    pub gross: Decimal,
    pub margin: Decimal,
    pub net: Decimal,
    pub orders_caption: String,
}

/// What the service sheet's automatic checks found.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Checks {
    pub missing_cost: usize,
    pub gross_consistent: bool,
    pub revenue_consistent: bool,
}

/// A report's whole set of figures, ready to be drawn.
pub(crate) struct ReportFigures {
    pub period: DayRange,
    /// The business the report is issued by.
    pub company: String,
    /// The line under every sheet's title.
    pub subtitle: String,
    pub kpi: Kpi,
    pub waterfall: Vec<WaterfallRow>,
    pub orders: Vec<OrderFigure>,
    pub expenses: Vec<ExpenseFigure>,
    pub categories: Vec<CategoryFigure>,
    pub totals: Totals,
    pub checks: Checks,
    /// The three bars of the comparison chart: revenue, gross, net result.
    pub chart: [Decimal; 3],
}

impl ReportFigures {
    pub fn new(context: &ReportContext, period: DayRange, sources: &ReportSources) -> Self {
        let priced = price_orders(sources);
        let parties: HashMap<&str, &OrderParty> = sources
            .parties
            .iter()
            .map(|party| (party.order_id.as_str(), party))
            .collect();

        // The query already returns them in this order; sorting here keeps the
        // workbook the same whatever order the rows come back in.
        let mut orders: Vec<&WorkOrder> = sources.orders.iter().collect();
        orders.sort_by(|left, right| {
            left.paid_at
                .cmp(&right.paid_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        let mut totals = Totals::default();
        let mut order_rows = Vec::with_capacity(orders.len());

        for order in orders {
            let Some(priced_order) = priced.get(&order.id) else {
                continue;
            };

            let has_billed_parts = priced_order
                .part_items
                .iter()
                .any(|dto| part_is_billed(dto.item.status, dto.item.is_free, dto.item.provided_by));

            // Hourly cost tracking has no hours to measure a flat-fee item by,
            // so such an order's labor cost is only partly known.
            let labor_cost_known = priced_order
                .labor_items
                .iter()
                .filter(|dto| labor_is_billed(dto.item.is_free))
                .all(|dto| labor_cost_basis_known(dto.item.price_type, order.labor_cost_type));

            let money = order_money(&priced_order.totals, has_billed_parts, labor_cost_known);

            let party = parties.get(order.id.as_str()).copied();
            let (client, car) = match party {
                Some(party) => (
                    client_display_name(
                        party.client_type,
                        &party.client_name,
                        party.client_surname.as_deref(),
                        party.client_first_name.as_deref(),
                        party.client_patronymic.as_deref(),
                    ),
                    car_display_name(party.car_brand.as_deref(), &party.car_model),
                ),
                // The order exists, so its car does too; only a client that
                // somehow lost its row can end up here.
                None => ("—".to_string(), "—".to_string()),
            };

            let paid_on = order
                .paid_at
                .map(|paid_at| paid_at.with_timezone(&context.timezone).date_naive())
                .unwrap_or(period.start);

            totals.revenue += money.revenue;
            totals.labor += money.labor;
            totals.parts += money.parts;
            totals.discount += money.discount;
            totals.labor_cost += money.labor_cost.unwrap_or(Decimal::ZERO);
            totals.parts_cost += money.parts_cost.unwrap_or(Decimal::ZERO);
            totals.gross += money.gross;
            totals.order_count += 1;

            if money.labor_cost.is_none() || money.parts_cost.is_none() {
                totals.missing_cost_count += 1;
            }

            order_rows.push(OrderFigure {
                id: order.id.clone(),
                paid_on,
                payment: payment_name(order.payment_method),
                client,
                car,
                labor: money.labor,
                parts: money.parts,
                discount: money.discount,
                revenue: money.revenue,
                labor_cost: money.labor_cost,
                parts_cost: money.parts_cost,
                gross: money.gross,
                margin: money.margin,
            });
        }

        // Categories keep the order they are first met in, the way the period's
        // expenses read down the page.
        let mut categories: Vec<(String, Decimal)> = Vec::new();
        let mut expense_rows = Vec::with_capacity(sources.expenses.len());

        for expense in &sources.expenses {
            let amount = pricing::net_kept_amount(
                expense.amount,
                expense.vat_rate,
                expense.price_includes_vat,
            );
            let category = expense
                .category
                .as_ref()
                .map(|category| category.name.trim().to_string())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "Без категорії".to_string());

            totals.expenses += amount;
            match categories.iter_mut().find(|(name, _)| name == &category) {
                Some(entry) => entry.1 += amount,
                None => categories.push((category.clone(), amount)),
            }

            expense_rows.push(ExpenseFigure {
                date: NaiveDate::parse_from_str(&expense.date, "%Y-%m-%d").ok(),
                date_text: expense.date.clone(),
                description: expense.description.trim().to_string(),
                category,
                amount,
                vat_rate: expense.vat_rate,
                payment: payment_name(expense.payment_method),
            });
        }

        totals.net = totals.gross - totals.expenses;
        totals.margin = ratio(totals.gross, totals.revenue);

        let categories: Vec<CategoryFigure> = categories
            .into_iter()
            .map(|(name, amount)| CategoryFigure {
                name,
                amount,
                share: ratio(amount, totals.expenses),
            })
            .collect();

        let percent = |value: Decimal| ratio(value, totals.revenue);
        let negative = |value: Decimal| -value;

        let discounts = negative(totals.discount);

        // The gross income split is shown without a discount line of its own,
        // so each side's share of the period's discount comes off it first.
        // The split follows the revenue the side brought in; the remainder
        // goes to the parts so the two lines add up to the gross exactly.
        let revenue_lines = totals.labor + totals.parts;
        let labor_discount = (totals.discount * ratio(totals.labor, revenue_lines)).round_dp(2);
        let parts_discount = totals.discount - labor_discount;
        let services_gross = totals.labor - totals.labor_cost - labor_discount;
        let parts_gross = totals.parts - totals.parts_cost - parts_discount;

        let waterfall = vec![
            WaterfallRow {
                label: "Робота",
                amount: totals.labor,
                percent: percent(totals.labor),
                kind: WaterfallKind::Plain,
            },
            WaterfallRow {
                label: "Запчастини",
                amount: totals.parts,
                percent: percent(totals.parts),
                kind: WaterfallKind::Plain,
            },
            WaterfallRow {
                label: "Знижки",
                amount: discounts,
                percent: percent(discounts),
                kind: WaterfallKind::Plain,
            },
            WaterfallRow {
                label: "Виручка",
                amount: totals.revenue,
                percent: percent(totals.revenue),
                kind: WaterfallKind::Subtotal,
            },
            WaterfallRow {
                label: "Собівартість роботи",
                amount: negative(totals.labor_cost),
                percent: percent(negative(totals.labor_cost)),
                kind: WaterfallKind::Plain,
            },
            WaterfallRow {
                label: "Собівартість запчастин",
                amount: negative(totals.parts_cost),
                percent: percent(negative(totals.parts_cost)),
                kind: WaterfallKind::Plain,
            },
            WaterfallRow {
                label: "Валовий дохід",
                amount: totals.gross,
                percent: percent(totals.gross),
                kind: WaterfallKind::Gross,
            },
            // The gross income split, indented under the line it breaks down.
            WaterfallRow {
                label: "у т.ч. послуги",
                amount: services_gross,
                percent: percent(services_gross),
                kind: WaterfallKind::Breakdown,
            },
            WaterfallRow {
                label: "у т.ч. запчастини",
                amount: parts_gross,
                percent: percent(parts_gross),
                kind: WaterfallKind::Breakdown,
            },
            WaterfallRow {
                label: "Витрати поза замовленнями",
                amount: negative(totals.expenses),
                percent: percent(negative(totals.expenses)),
                kind: WaterfallKind::Plain,
            },
            WaterfallRow {
                label: "Чистий результат",
                amount: totals.net,
                percent: percent(totals.net),
                kind: WaterfallKind::Final,
            },
        ];

        // The two "у т.ч." lines must add back up to the gross income, and the
        // revenue lines to the revenue. They are built from the same sums, so
        // this only fails if one of the two ever stops adding up.
        let breakdown_gross = services_gross + parts_gross;
        let checks = Checks {
            missing_cost: totals.missing_cost_count,
            gross_consistent: breakdown_gross == totals.gross,
            revenue_consistent: totals.labor + totals.parts + discounts == totals.revenue,
        };

        let kpi = Kpi {
            revenue: totals.revenue,
            gross: totals.gross,
            margin: totals.margin,
            net: totals.net,
            orders_caption: format!(
                "{} {}",
                totals.order_count,
                plural(
                    totals.order_count,
                    "оплачене замовлення",
                    "оплачені замовлення",
                    "оплачених замовлень",
                )
            ),
        };

        let subtitle = format!(
            "{}   ·   Період: {} – {}   ·   Сформовано: {}",
            context.company,
            period.start.format("%d.%m.%Y"),
            period.end.format("%d.%m.%Y"),
            context
                .generated_at
                .with_timezone(&context.timezone)
                .format("%d.%m.%Y %H:%M")
        );

        Self {
            period,
            company: context.company.clone(),
            subtitle,
            kpi,
            waterfall,
            orders: order_rows,
            expenses: expense_rows,
            categories,
            totals,
            checks,
            chart: [totals.revenue, totals.gross, totals.net],
        }
    }
}

/// The money one order contributes to the report.
struct OrderMoney {
    labor: Decimal,
    parts: Decimal,
    discount: Decimal,
    revenue: Decimal,
    labor_cost: Option<Decimal>,
    parts_cost: Option<Decimal>,
    gross: Decimal,
    margin: Decimal,
}

/// Net revenue for an order and how it is made up.
///
/// Work and parts are taken at what they were billed before any discount, the
/// discount is everything taken off them, and revenue is the net figure the
/// order records. The discount is derived from the other three, so the lines
/// always add back up to the revenue even where VAT extraction rounded a cent
/// away.
fn order_money(
    totals: &WorkOrderTotals,
    has_billed_parts: bool,
    labor_cost_known: bool,
) -> OrderMoney {
    // The lines are already net of their own VAT and before the order
    // discount, so the discount is what is left between them and the base.
    let labor = totals.labor_net;
    let parts = totals.parts_net;
    let revenue = totals.vat_base;
    let discount = labor + parts - revenue;

    // An order that logs cost by the hour has no hours for its flat-fee work,
    // so the labor cost it records is only part of the truth. The sheet says so
    // rather than showing a cost that is quietly too low.
    let labor_cost = if labor_cost_known {
        Some(totals.labor_cost)
    } else {
        None
    };

    // A part with no purchase price bills at its markup alone. The shop cannot
    // tell that apart from a genuinely free part, so the order says so and the
    // sheet warns the gross may be overstated.
    let parts_cost = if has_billed_parts && totals.parts_cost.is_zero() {
        None
    } else {
        Some(totals.parts_cost)
    };

    let cost = labor_cost.unwrap_or(Decimal::ZERO) + parts_cost.unwrap_or(Decimal::ZERO);
    let gross = revenue - cost;

    OrderMoney {
        labor,
        parts,
        discount,
        revenue,
        labor_cost,
        parts_cost,
        gross,
        margin: ratio(gross, revenue),
    }
}

/// Prices every order of the period with the same rules the order screen uses.
struct PricedOrder {
    totals: WorkOrderTotals,
    labor_items: Vec<LaborItemDto>,
    part_items: Vec<PartItemDto>,
}

fn price_orders(sources: &ReportSources) -> HashMap<String, PricedOrder> {
    let mut labor: HashMap<&str, Vec<&LaborItem>> = HashMap::new();
    for item in &sources.labor_items {
        labor
            .entry(item.work_order_id.as_str())
            .or_default()
            .push(item);
    }

    let mut parts: HashMap<&str, Vec<&PartItem>> = HashMap::new();
    for item in &sources.part_items {
        parts
            .entry(item.work_order_id.as_str())
            .or_default()
            .push(item);
    }

    let mut priced = HashMap::with_capacity(sources.orders.len());
    for order in &sources.orders {
        let labor_items: Vec<LaborItemDto> = labor
            .get(order.id.as_str())
            .map(|items| {
                items
                    .iter()
                    .map(|item| LaborItemDto {
                        item: (*item).clone(),
                        totals: pricing::calculate_labor_item_totals(
                            item,
                            order.rate,
                            order.labor_cost_type,
                            order.labor_cost_value,
                            order.vat_rate,
                            order.vat_scheme,
                            order.prices_include_vat,
                        ),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let part_items: Vec<PartItemDto> = parts
            .get(order.id.as_str())
            .map(|items| {
                items
                    .iter()
                    .map(|item| PartItemDto {
                        item: (*item).clone(),
                        totals: pricing::calculate_part_item_totals(
                            item,
                            order.vat_rate,
                            order.vat_scheme,
                            order.prices_include_vat,
                        ),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let totals = pricing::calculate_order_totals(order, &labor_items, &part_items);

        priced.insert(
            order.id.clone(),
            PricedOrder {
                totals,
                labor_items,
                part_items,
            },
        );
    }

    priced
}

/// `part / whole`, or zero when there is no whole to divide by.
fn ratio(part: Decimal, whole: Decimal) -> Decimal {
    if whole.is_zero() {
        Decimal::ZERO
    } else {
        part / whole
    }
}

/// The payment method as the report names it, matching the app's labels.
fn payment_name(method: Option<PaymentMethod>) -> String {
    match method {
        Some(PaymentMethod::Cash) => "Готівка",
        Some(PaymentMethod::Card) => "Картка",
        Some(PaymentMethod::BankTransfer) => "Безготівковий (рахунок)",
        Some(PaymentMethod::CardTransfer) => "Переказ на картку",
        None => "—",
    }
    .to_string()
}

/// The company the report is issued by.
fn company_name(settings: &AppSettings) -> String {
    let Some(info) = settings.company_info.as_ref() else {
        return "Дані бізнесу не заповнено".to_string();
    };
    let name = match info.kind {
        CompanyKind::Fop => full_name(info),
        CompanyKind::Company => info
            .name
            .clone()
            .or_else(|| info.legal_name.clone())
            .unwrap_or_default(),
    };
    let name = name.trim().to_string();
    if name.is_empty() {
        "Дані бізнесу не заповнено".to_string()
    } else {
        name
    }
}

fn full_name(info: &CompanyInfo) -> String {
    [&info.surname, &info.first_name, &info.patronymic]
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}
