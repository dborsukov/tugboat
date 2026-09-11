use rust_decimal::Decimal;

use super::structs::{DiscountType, LaborItem, PartItem, PartStatus};
use crate::database::work_orders::{
    dtos::{LaborItemTotals, PartItemTotals, WorkOrderTotals},
    structs::{LaborCostType, LaborPriceType},
};

/// A part counts toward the current order total only once it is actually
/// installed and billed: status `Installed` and not marked free.
pub fn part_is_billed(status: PartStatus, is_free: bool) -> bool {
    !is_free && status == PartStatus::Installed
}

pub fn part_is_pending(status: PartStatus, is_free: bool) -> bool {
    !is_free
        && matches!(
            status,
            PartStatus::Quoted | PartStatus::Ordered | PartStatus::InStock | PartStatus::Installing
        )
}

pub fn part_is_projected_billed(status: PartStatus, is_free: bool) -> bool {
    part_is_billed(status, is_free) || part_is_pending(status, is_free)
}

pub fn labor_is_billed(is_free: bool) -> bool {
    !is_free
}

pub fn net_kept_amount(gross: Decimal, vat_rate: Decimal, prices_include_vat: bool) -> Decimal {
    if !prices_include_vat || vat_rate.is_zero() {
        return gross;
    }
    (gross * (Decimal::ONE_HUNDRED / (Decimal::ONE_HUNDRED + vat_rate))).round_dp(2)
}

pub fn labor_cost_basis_known(price_type: LaborPriceType, labor_cost_type: LaborCostType) -> bool {
    !matches!(labor_cost_type, LaborCostType::Hourly)
        || matches!(price_type, LaborPriceType::Hourly)
}

pub fn calculate_labor_item_totals(
    labor_item: &LaborItem,
    rate: Decimal,
    labor_cost_type: LaborCostType,
    labor_cost: Decimal,
    vat_rate: Decimal,
    prices_include_vat: bool,
) -> LaborItemTotals {
    let price = match labor_item.price_type {
        LaborPriceType::Hourly => rate.round_dp(2),
        LaborPriceType::Fixed => labor_item.fixed_price.round_dp(2),
    };

    let discounted_price = match labor_item.discount_type {
        DiscountType::None => price,
        DiscountType::Fixed => price - labor_item.discount_value,
        DiscountType::Percentage => {
            (price * (Decimal::ONE - labor_item.discount_value / Decimal::ONE_HUNDRED)).round_dp(2)
        }
    }
    .max(Decimal::ZERO);

    let total = match labor_item.price_type {
        LaborPriceType::Hourly => (discounted_price * labor_item.labor_hours).round_dp(2),
        LaborPriceType::Fixed => discounted_price,
    };

    let discount = match labor_item.price_type {
        LaborPriceType::Hourly => ((price - discounted_price) * labor_item.labor_hours)
            .round_dp(2)
            .max(Decimal::ZERO),
        LaborPriceType::Fixed => (price - discounted_price).max(Decimal::ZERO),
    };

    let gross = match labor_item.price_type {
        LaborPriceType::Hourly => (price * labor_item.labor_hours).round_dp(2),
        LaborPriceType::Fixed => price,
    };

    // When labor_cost_type is Hourly, cost is derived from labor_hours.
    // For a Fixed-priced item, labor_hours is always zero (it's not tracked
    // independently of billing), so cost intentionally comes out to 0 here.
    // There is no hours-based cost basis for a flat-fee job. This is not a
    // bug, it reflects that Hourly cost tracking has nothing to measure
    // against for items that were never billed by the hour. Callers that
    // surface cost to users should treat this combination (Hourly cost
    // type + Fixed price_type) as "no cost basis available", not as true 0.
    let cost = match labor_cost_type {
        LaborCostType::Hourly => (labor_item.labor_hours * labor_cost).round_dp(2),
        LaborCostType::Percentage => (gross * (labor_cost / Decimal::ONE_HUNDRED)).round_dp(2),
    };

    let cost_basis_known = labor_cost_basis_known(labor_item.price_type, labor_cost_type);

    let below_cost = !labor_item.is_free
        && cost_basis_known
        && net_kept_amount(total, vat_rate, prices_include_vat) < cost;

    LaborItemTotals {
        cost,
        price,
        discounted_price,
        discount,
        total,
        below_cost,
    }
}

pub fn calculate_part_item_totals(
    part_item: &PartItem,
    vat_rate: Decimal,
    prices_include_vat: bool,
) -> PartItemTotals {
    let cost = (part_item.base_cost * part_item.quantity).round_dp(2);

    let retail_price = (part_item.base_cost
        * (Decimal::ONE + part_item.markup / Decimal::ONE_HUNDRED))
        .round_dp(2);

    let discounted_price = match part_item.discount_type {
        DiscountType::None => retail_price,
        DiscountType::Fixed => retail_price - part_item.discount_value,
        DiscountType::Percentage => (retail_price
            * (Decimal::ONE - part_item.discount_value / Decimal::ONE_HUNDRED))
            .round_dp(2),
    };

    let retail_total = (retail_price * part_item.quantity).round_dp(2);
    let total = (discounted_price * part_item.quantity)
        .round_dp(2)
        .max(Decimal::ZERO);

    let discount = (retail_total - total).max(Decimal::ZERO);

    let below_cost = !part_item.is_free
        && net_kept_amount(discounted_price, vat_rate, prices_include_vat) < part_item.base_cost;

    PartItemTotals {
        cost,
        retail_price,
        discounted_price,
        discount,
        total,
        below_cost,
    }
}

/// Computes the VAT breakdown for an order total (after discounts).
///
/// `prices_include_vat` selects how the entered prices relate to the tax:
/// - `false` (prices without VAT): the base is the discounted total and VAT
///   is added on top (`base * rate / 100`).
/// - `true` (prices already include VAT): the discounted total is the
///   VAT-inclusive amount, so VAT is extracted with the reverse rate
///   (`total * rate / (100 + rate)`).
///
/// A zero rate yields zero VAT on both paths.
pub fn calculate_vat(
    total: Decimal,
    vat_rate: Decimal,
    prices_include_vat: bool,
) -> (Decimal, Decimal, Decimal) {
    let (vat_base, vat_amount, total_with_vat) = if prices_include_vat {
        let vat_amount = (total * (vat_rate / (Decimal::ONE_HUNDRED + vat_rate))).round_dp(2);
        let vat_base = (total - vat_amount).round_dp(2);
        (vat_base, vat_amount, total)
    } else {
        let vat_base = total;
        let vat_amount = (total * (vat_rate / Decimal::ONE_HUNDRED)).round_dp(2);
        (vat_base, vat_amount, (total + vat_amount).round_dp(2))
    };
    (vat_base, vat_amount, total_with_vat)
}

#[allow(clippy::too_many_arguments)]
pub fn calculate_work_order_totals(
    labor_total: Decimal,
    labor_cost: Decimal,
    parts_total: Decimal,
    parts_cost: Decimal,
    parts_pending_total: Decimal,
    labor_discount: Decimal,
    parts_discount: Decimal,
    discount_type: DiscountType,
    discount_value: Decimal,
    vat_rate: Decimal,
    prices_include_vat: bool,
) -> WorkOrderTotals {
    let subtotal = labor_total + parts_total;

    let grand_total = match discount_type {
        DiscountType::None => subtotal,
        DiscountType::Fixed => subtotal - discount_value,
        DiscountType::Percentage => {
            (subtotal * (Decimal::ONE - discount_value / Decimal::ONE_HUNDRED)).round_dp(2)
        }
    };
    let grand_total = grand_total.max(Decimal::ZERO);

    let order_discount = (subtotal - grand_total).max(Decimal::ZERO);

    let (vat_base, vat_amount, total_with_vat) =
        calculate_vat(grand_total, vat_rate, prices_include_vat);

    let below_cost = vat_base < labor_cost + parts_cost;

    WorkOrderTotals {
        labor_total,
        labor_cost,
        parts_total,
        parts_cost,
        parts_pending_total,
        subtotal,
        labor_discount,
        parts_discount,
        order_discount,
        grand_total,
        vat_base,
        vat_amount,
        total_with_vat,
        below_cost,
    }
}
