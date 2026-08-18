use rust_decimal::Decimal;

use super::structs::{DiscountType, LaborItem, PartItem, WorkOrder};
use crate::database::work_orders::dtos::{LaborItemTotals, PartItemTotals, WorkOrderTotals};

pub fn calculate_labor_item_totals(labor_item: &LaborItem, rate: Decimal) -> LaborItemTotals {
    let subtotal = (labor_item.labor_hours * rate).round_dp(2);

    let total = match labor_item.discount_type {
        DiscountType::None => subtotal,
        DiscountType::Fixed => subtotal - labor_item.discount_amount,
        DiscountType::Percentage => (subtotal
            * (Decimal::ONE - labor_item.discount_percent / Decimal::ONE_HUNDRED))
            .round_dp(2),
    };
    let total = total.max(Decimal::ZERO);

    // Derived from subtotal - total (rather than discount_amount/discount_percent
    // directly) so it always reflects what was actually deducted, including cases
    // where the raw discount would have pushed the total below zero.
    let discount = (subtotal - total).max(Decimal::ZERO);

    LaborItemTotals {
        // TODO: figure out how to implement labor costs
        // No logic for labor cost, copy subtotal for now
        cost: subtotal,
        subtotal,
        discount,
        total,
    }
}

pub fn calculate_part_item_totals(part_item: &PartItem) -> PartItemTotals {
    let cost = (part_item.base_cost * part_item.quantity).round_dp(2);

    let retail_price = (part_item.base_cost
        * (Decimal::ONE + part_item.markup / Decimal::ONE_HUNDRED))
        .round_dp(2);

    let discounted_price = match part_item.discount_type {
        DiscountType::None => retail_price,
        DiscountType::Fixed => retail_price - part_item.discount_amount,
        DiscountType::Percentage => (retail_price
            * (Decimal::ONE - part_item.discount_percent / Decimal::ONE_HUNDRED))
            .round_dp(2),
    };

    let retail_total = (retail_price * part_item.quantity).round_dp(2);
    let total = (discounted_price * part_item.quantity)
        .round_dp(2)
        .max(Decimal::ZERO);

    // Quantity-adjusted, so it lines up with `total` (also quantity-adjusted)
    // rather than the per-unit retail_price/discounted_price fields.
    let discount = (retail_total - total).max(Decimal::ZERO);

    PartItemTotals {
        cost,
        retail_price,
        discounted_price,
        discount,
        total,
    }
}

pub fn calculate_work_order_totals(
    work_order: &WorkOrder,
    labor_total: Decimal,
    labor_cost: Decimal,
    parts_total: Decimal,
    parts_cost: Decimal,
    labor_discount: Decimal,
    parts_discount: Decimal,
) -> WorkOrderTotals {
    let subtotal = labor_total + parts_total;

    let grand_total = match work_order.discount_type {
        DiscountType::None => subtotal,
        DiscountType::Fixed => subtotal - work_order.discount_amount,
        DiscountType::Percentage => (subtotal
            * (Decimal::ONE - work_order.discount_percent / Decimal::ONE_HUNDRED))
            .round_dp(2),
    };
    let grand_total = grand_total.max(Decimal::ZERO);

    // The work order's own discount, on top of whatever was already taken off
    // individual labor/part items.
    let order_discount = (subtotal - grand_total).max(Decimal::ZERO);

    WorkOrderTotals {
        labor_total,
        labor_cost,
        parts_total,
        parts_cost,
        subtotal,
        labor_discount,
        parts_discount,
        order_discount,
        grand_total,
    }
}
