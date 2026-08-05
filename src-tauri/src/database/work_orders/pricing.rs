use super::structs::{DiscountType, WorkOrder, WorkOrderLaborItem, WorkOrderPartItem};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkOrderTotals {
    /// Sum of labor lines: labor_hours × rate_cents, each line rounded.
    pub labor_total_cents: i64,
    /// Sum of part lines: quantity × cost_cents × (1 + markup%), each line rounded.
    pub parts_total_cents: i64,
    /// labor_total_cents + parts_total_cents, before any discount.
    pub subtotal_cents: i64,
    /// subtotal_cents with the work order's discount applied.
    pub grand_total_cents: i64,
}

pub fn calculate_totals(
    work_order: &WorkOrder,
    labor_items: &[&WorkOrderLaborItem],
    part_items: &[&WorkOrderPartItem],
) -> WorkOrderTotals {
    let labor_total_cents: i64 = labor_items
        .iter()
        .map(|item| (item.labor_hours * work_order.rate_cents as f64).round() as i64)
        .sum();

    let parts_total_cents: i64 = part_items
        .iter()
        .map(|item| {
            (item.quantity * (item.cost_cents as f64 * (1.0 + item.markup / 100.0))).round() as i64
        })
        .sum();

    let subtotal_cents = labor_total_cents + parts_total_cents;

    let grand_total_cents = if work_order.discount_cents != 0 || work_order.discount_percent != 0.0
    {
        match work_order.discount_type {
            DiscountType::Fixed => subtotal_cents - work_order.discount_cents,
            DiscountType::Percentage => {
                (subtotal_cents as f64 * (1.0 - work_order.discount_percent / 100.0)).round() as i64
            }
        }
    } else {
        subtotal_cents
    };

    WorkOrderTotals {
        labor_total_cents,
        parts_total_cents,
        subtotal_cents,
        grand_total_cents,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn work_order(
        rate_cents: i64,
        discount_type: DiscountType,
        discount_cents: i64,
        discount_percent: f64,
    ) -> WorkOrder {
        WorkOrder {
            id: "wo_1".into(),
            car_id: 1,
            odometer_in: 0,
            customer_complaint: None,
            mechanic_notes: None,
            status: crate::database::work_orders::structs::WorkOrderStatus::Received,
            rate_cents,
            discount_type,
            discount_cents,
            discount_percent,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            paid_at: None,
            payment_method: None,
        }
    }

    #[test]
    fn no_discount_sums_rounded_lines() {
        let wo = work_order(150_00, DiscountType::Fixed, 0, 0.0);
        let labor = WorkOrderLaborItem {
            id: 1,
            work_order_id: "wo_1".into(),
            service_id: None,
            code: None,
            name: "Oil change".into(),
            labor_hours: 0.5,
        };
        let part = WorkOrderPartItem {
            id: 1,
            work_order_id: "wo_1".into(),
            code: None,
            name: "Oil filter".into(),
            quantity: 1.0,
            unit_type: "unit".into(),
            cost_cents: 500,
            markup: 20.0,
            status: crate::database::work_orders::structs::PartStatus::Installed,
        };
        let totals = calculate_totals(&wo, &[&labor], &[&part]);
        // labor: round(0.5 * 15000) = 7500, part: round(1 * (500 * 1.2)) = 600
        assert_eq!(totals.labor_total_cents, 7500);
        assert_eq!(totals.parts_total_cents, 600);
        assert_eq!(totals.subtotal_cents, 8100);
        assert_eq!(totals.grand_total_cents, 8100);
    }

    #[test]
    fn fixed_discount_subtracted_from_subtotal() {
        let wo = work_order(100_00, DiscountType::Fixed, 500, 0.0);
        let totals = calculate_totals(&wo, &[], &[]);
        assert_eq!(totals.subtotal_cents, 0);
        assert_eq!(totals.grand_total_cents, -500);
    }

    #[test]
    fn percentage_discount_applied_to_subtotal() {
        let wo = work_order(100_00, DiscountType::Percentage, 0, 10.0);
        let labor = WorkOrderLaborItem {
            id: 1,
            work_order_id: "wo_1".into(),
            service_id: None,
            code: None,
            name: "Labor".into(),
            labor_hours: 1.0,
        };
        let totals = calculate_totals(&wo, &[&labor], &[]);
        // subtotal = 10000, minus 10% = 9000
        assert_eq!(totals.subtotal_cents, 10000);
        assert_eq!(totals.grand_total_cents, 9000);
    }
}
