use super::structs::{DiscountType, LaborItem, PartItem, WorkOrder};
use crate::database::work_orders::dtos::{LaborItemTotals, PartItemTotals, WorkOrderTotals};

pub fn calculate_labor_item_totals(labor_item: &LaborItem, rate_cents: i64) -> LaborItemTotals {
    let subtotal_cents = (labor_item.labor_hours * rate_cents as f64).round() as i64;

    let total_cents = match labor_item.discount_type {
        DiscountType::Fixed => subtotal_cents - labor_item.discount_cents,
        DiscountType::Percentage => {
            (subtotal_cents as f64 * (1.0 - labor_item.discount_percent / 100.0)).round() as i64
        }
    };

    LaborItemTotals {
        subtotal_cents,
        total_cents: total_cents.max(0),
    }
}

pub fn calculate_part_item_totals(part_item: &PartItem) -> PartItemTotals {
    let retail_price_cents =
        (part_item.base_cost_cents as f64 * (1.0 + part_item.markup / 100.0)).round() as i64;

    let discounted_price_cents = match part_item.discount_type {
        DiscountType::Fixed => retail_price_cents - part_item.discount_cents,
        DiscountType::Percentage => {
            (retail_price_cents as f64 * (1.0 - part_item.discount_percent / 100.0)).round() as i64
        }
    };

    let total_cents = (discounted_price_cents as f64 * part_item.quantity).round() as i64;

    PartItemTotals {
        retail_price_cents,
        discounted_price_cents,
        total_cents: total_cents.max(0),
    }
}

pub fn calculate_work_order_totals(
    work_order: &WorkOrder,
    labor_total_cents: i64,
    parts_total_cents: i64,
) -> WorkOrderTotals {
    let subtotal_cents = labor_total_cents + parts_total_cents;

    let grand_total_cents = match work_order.discount_type {
        DiscountType::Fixed => subtotal_cents - work_order.discount_cents,
        DiscountType::Percentage => {
            (subtotal_cents as f64 * (1.0 - work_order.discount_percent / 100.0)).round() as i64
        }
    };

    WorkOrderTotals {
        labor_total_cents,
        parts_total_cents,
        subtotal_cents,
        grand_total_cents: grand_total_cents.max(0),
    }
}
