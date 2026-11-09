use std::collections::BTreeMap;

use rust_decimal::Decimal;

use super::structs::{DiscountType, LaborItem, PartItem, PartStatus, ProvidedBy, WorkOrder};
use crate::database::{
    vat::structs::VatScheme,
    work_orders::{
        dtos::{
            LaborItemDto, LaborItemTotals, PartItemDto, PartItemTotals, VatBucket, WorkOrderTotals,
        },
        structs::{LaborCostType, LaborPriceType},
    },
};

/// A part counts toward the current order total only once it is actually
/// installed and billed: status `Installed` and not marked free.
pub fn part_is_billed(status: PartStatus, is_free: bool, provided_by: ProvidedBy) -> bool {
    provided_by == ProvidedBy::Shop && !is_free && status == PartStatus::Installed
}

pub fn part_is_pending(status: PartStatus, is_free: bool, provided_by: ProvidedBy) -> bool {
    provided_by == ProvidedBy::Shop
        && !is_free
        && matches!(
            status,
            PartStatus::Quoted | PartStatus::Ordered | PartStatus::InStock | PartStatus::Installing
        )
}

pub fn part_is_projected_billed(
    status: PartStatus,
    is_free: bool,
    provided_by: ProvidedBy,
) -> bool {
    part_is_billed(status, is_free, provided_by) || part_is_pending(status, is_free, provided_by)
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

pub fn line_vat(
    line_rate: Option<Decimal>,
    line_scheme: Option<VatScheme>,
    order_rate: Decimal,
    order_scheme: VatScheme,
) -> (Decimal, VatScheme) {
    (
        line_rate.unwrap_or(order_rate),
        line_scheme.unwrap_or(order_scheme),
    )
}

pub fn calculate_labor_item_totals(
    labor_item: &LaborItem,
    rate: Decimal,
    labor_cost_type: LaborCostType,
    labor_cost: Decimal,
    vat_rate: Decimal,
    vat_scheme: VatScheme,
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

    // The line may carry its own rate; the one passed in is the order's default.
    let (sale_rate, _) = line_vat(
        labor_item.vat_rate,
        labor_item.vat_scheme,
        vat_rate,
        vat_scheme,
    );

    let below_cost = !labor_item.is_free
        && cost_basis_known
        && net_kept_amount(total, sale_rate, prices_include_vat) < cost;

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
    vat_scheme: VatScheme,
    prices_include_vat: bool,
) -> PartItemTotals {
    // What the part cost the shop. When its purchase VAT is claimed, the
    // recoverable part is not a cost, margin compares net revenue to net cost.
    let unit_cost = if part_item.vat_credited && !part_item.purchase_vat_rate.is_zero() {
        net_kept_amount(
            part_item.base_cost,
            part_item.purchase_vat_rate,
            part_item.purchase_price_includes_vat,
        )
    } else {
        part_item.base_cost
    };
    let cost = (unit_cost * part_item.quantity).round_dp(2);

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

    let (sale_rate, _) = line_vat(
        part_item.vat_rate,
        part_item.vat_scheme,
        vat_rate,
        vat_scheme,
    );

    let below_cost = !part_item.is_free
        && net_kept_amount(discounted_price, sale_rate, prices_include_vat) < unit_cost;

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

/// Which table a line came from, so the report can split the revenue by side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineKind {
    Labor,
    Parts,
}

/// One billed line as VAT sees it: what it was billed before the order
/// discount, and the rate and treatment it is billed at.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PricedLine {
    pub kind: LineKind,
    /// The line after its own discount, before the order discount.
    pub total: Decimal,
    /// The line before its own discount, used for the revenue split so the
    /// discount stays a separate line in the report.
    pub retail: Decimal,
    pub rate: Decimal,
    pub scheme: VatScheme,
}

/// Splits the period's VAT into one bucket per rate and treatment.
///
/// The order discount is shared out over the billed lines in proportion to
/// what each was billed, so every line's taxable base reflects the discount.
/// VAT is then rounded once per bucket rather than once per order, which keeps
/// a single-rate order identical to what it was before rates could differ.
///
/// Returns the buckets, the totals derived from them, and the work and parts
/// revenue before the order discount, net of VAT.
fn allocate_vat(
    lines: &[PricedLine],
    subtotal: Decimal,
    grand_total: Decimal,
    prices_include_vat: bool,
) -> (Vec<VatBucket>, Decimal, Decimal, Decimal, Decimal, Decimal) {
    if lines.is_empty() {
        return (
            Vec::new(),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );
    }

    // Revenue lines before the order discount, net of VAT. For a single-rate
    // order this is exactly the old side netting. It is computed even when the
    // discounted total is zero: a fully discounted order still has lines and a
    // cost to report.
    let mut labor_net = Decimal::ZERO;
    let mut parts_net = Decimal::ZERO;
    for line in lines {
        let net = net_kept_amount(line.retail, line.rate, prices_include_vat);
        match line.kind {
            LineKind::Labor => labor_net += net,
            LineKind::Parts => parts_net += net,
        }
    }

    if subtotal.is_zero() {
        return (
            Vec::new(),
            Decimal::ZERO,
            Decimal::ZERO,
            grand_total,
            labor_net,
            parts_net,
        );
    }

    // Share the order discount out over the lines. The residual of rounding is
    // put on the largest line so the allocated amounts add up to the total.
    let ratio = grand_total / subtotal;
    let mut allocated: Vec<Decimal> = lines
        .iter()
        .map(|line| (line.total * ratio).round_dp(2))
        .collect();
    let allocated_sum: Decimal = allocated.iter().sum();
    let residual = grand_total - allocated_sum;
    if !residual.is_zero() {
        if let Some((index, _)) = allocated
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.cmp(right))
        {
            allocated[index] += residual;
        }
    }

    // Sum what each rate and treatment was billed, then round the VAT once for
    // the bucket.
    let mut bucket_totals: BTreeMap<(Decimal, VatScheme), Decimal> = BTreeMap::new();
    for (line, amount) in lines.iter().zip(allocated) {
        *bucket_totals
            .entry((line.rate, line.scheme))
            .or_insert(Decimal::ZERO) += amount;
    }

    let buckets: Vec<VatBucket> = bucket_totals
        .into_iter()
        .map(|((rate, scheme), amount)| {
            let (base, vat, _) = calculate_vat(amount, rate, prices_include_vat);
            VatBucket {
                rate,
                scheme,
                base,
                amount: vat,
            }
        })
        .collect();

    let vat_base: Decimal = buckets.iter().map(|bucket| bucket.base).sum();
    let vat_amount: Decimal = buckets.iter().map(|bucket| bucket.amount).sum();
    let total_with_vat: Decimal = buckets
        .iter()
        .map(|bucket| bucket.base + bucket.amount)
        .sum();

    (
        buckets,
        vat_base,
        vat_amount,
        total_with_vat,
        labor_net,
        parts_net,
    )
}

/// Rolls already priced line items up into the totals of a whole order.
///
/// This is the single place where item level totals become order level ones:
/// the order detail view and the analytics dashboard both go through it, so a
/// figure on the dashboard can never disagree with the same figure on the order screen.
pub fn calculate_order_totals(
    work_order: &WorkOrder,
    labor_items: &[LaborItemDto],
    part_items: &[PartItemDto],
) -> WorkOrderTotals {
    let billed_labor = || {
        labor_items
            .iter()
            .filter(|dto| labor_is_billed(dto.item.is_free))
    };
    let billed_parts = || {
        part_items
            .iter()
            .filter(|dto| part_is_billed(dto.item.status, dto.item.is_free, dto.item.provided_by))
    };

    let labor_total: Decimal = billed_labor().map(|dto| dto.totals.total).sum();
    let labor_cost: Decimal = billed_labor().map(|dto| dto.totals.cost).sum();
    let labor_discount: Decimal = billed_labor().map(|dto| dto.totals.discount).sum();
    let parts_total: Decimal = billed_parts().map(|dto| dto.totals.total).sum();
    let parts_cost: Decimal = billed_parts().map(|dto| dto.totals.cost).sum();
    let parts_discount: Decimal = billed_parts().map(|dto| dto.totals.discount).sum();
    let parts_pending_total: Decimal = part_items
        .iter()
        .filter(|dto| part_is_pending(dto.item.status, dto.item.is_free, dto.item.provided_by))
        .map(|dto| dto.totals.total)
        .sum();

    let lines: Vec<PricedLine> = billed_labor()
        .map(|dto| {
            let (rate, scheme) = line_vat(
                dto.item.vat_rate,
                dto.item.vat_scheme,
                work_order.vat_rate,
                work_order.vat_scheme,
            );
            PricedLine {
                kind: LineKind::Labor,
                total: dto.totals.total,
                retail: dto.totals.total + dto.totals.discount,
                rate,
                scheme,
            }
        })
        .chain(billed_parts().map(|dto| {
            let (rate, scheme) = line_vat(
                dto.item.vat_rate,
                dto.item.vat_scheme,
                work_order.vat_rate,
                work_order.vat_scheme,
            );
            PricedLine {
                kind: LineKind::Parts,
                total: dto.totals.total,
                retail: dto.totals.total + dto.totals.discount,
                rate,
                scheme,
            }
        }))
        .collect();

    calculate_work_order_totals(
        labor_total,
        labor_cost,
        parts_total,
        parts_cost,
        parts_pending_total,
        labor_discount,
        parts_discount,
        work_order.discount_type,
        work_order.discount_value,
        work_order.prices_include_vat,
        &lines,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_work_order_totals(
    labor_total: Decimal,
    labor_cost: Decimal,
    parts_total: Decimal,
    parts_cost: Decimal,
    parts_pending_total: Decimal,
    labor_discount: Decimal,
    parts_discount: Decimal,
    discount_type: DiscountType,
    discount_value: Decimal,
    prices_include_vat: bool,
    lines: &[PricedLine],
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

    let (vat_buckets, vat_base, vat_amount, total_with_vat, labor_net, parts_net) =
        allocate_vat(lines, subtotal, grand_total, prices_include_vat);

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
        labor_net,
        parts_net,
        vat_base,
        vat_amount,
        total_with_vat,
        vat_buckets,
        below_cost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_kept_amount_strips_vat_only_when_prices_include_it() {
        assert_eq!(
            net_kept_amount(Decimal::from(120), Decimal::from(20), true),
            Decimal::from(100)
        );
        assert_eq!(
            net_kept_amount(Decimal::from(120), Decimal::from(20), false),
            Decimal::from(120)
        );
        assert_eq!(
            net_kept_amount(Decimal::from(120), Decimal::ZERO, true),
            Decimal::from(120)
        );
    }

    #[test]
    fn calculate_vat_splits_an_inclusive_total() {
        let (base, vat, total) = calculate_vat(Decimal::from(120), Decimal::from(20), true);
        assert_eq!(base, Decimal::from(100));
        assert_eq!(vat, Decimal::from(20));
        assert_eq!(total, Decimal::from(120));
    }

    #[test]
    fn calculate_vat_adds_vat_to_an_exclusive_total() {
        let (base, vat, total) = calculate_vat(Decimal::from(100), Decimal::from(20), false);
        assert_eq!(base, Decimal::from(100));
        assert_eq!(vat, Decimal::from(20));
        assert_eq!(total, Decimal::from(120));
    }

    #[test]
    fn zero_rate_yields_no_vat_on_either_path() {
        for prices_include_vat in [true, false] {
            let (_, vat, total) =
                calculate_vat(Decimal::from(100), Decimal::ZERO, prices_include_vat);
            assert!(vat.is_zero());
            assert_eq!(total, Decimal::from(100));
        }
    }

    #[test]
    fn only_installed_shop_parts_are_billed() {
        assert!(part_is_billed(
            PartStatus::Installed,
            false,
            ProvidedBy::Shop
        ));
        assert!(!part_is_billed(PartStatus::Quoted, false, ProvidedBy::Shop));
        assert!(!part_is_billed(
            PartStatus::Installed,
            true,
            ProvidedBy::Shop
        ));
        assert!(!part_is_billed(
            PartStatus::Installed,
            false,
            ProvidedBy::Customer
        ));
    }

    #[test]
    fn free_labor_is_not_billed() {
        assert!(labor_is_billed(false));
        assert!(!labor_is_billed(true));
    }
}
