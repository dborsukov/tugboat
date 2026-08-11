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

    LaborItemTotals {
        subtotal,
        total: total.max(Decimal::ZERO),
    }
}

pub fn calculate_part_item_totals(part_item: &PartItem) -> PartItemTotals {
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

    let total = (discounted_price * part_item.quantity).round_dp(2);

    PartItemTotals {
        retail_price,
        discounted_price,
        total: total.max(Decimal::ZERO),
    }
}

pub fn calculate_work_order_totals(
    work_order: &WorkOrder,
    labor_total: Decimal,
    parts_total: Decimal,
) -> WorkOrderTotals {
    let subtotal = labor_total + parts_total;

    let grand_total = match work_order.discount_type {
        DiscountType::None => subtotal,
        DiscountType::Fixed => subtotal - work_order.discount_amount,
        DiscountType::Percentage => (subtotal
            * (Decimal::ONE - work_order.discount_percent / Decimal::ONE_HUNDRED))
            .round_dp(2),
    };

    WorkOrderTotals {
        labor_total,
        parts_total,
        subtotal,
        grand_total: grand_total.max(Decimal::ZERO),
    }
}
