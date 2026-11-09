//! Bounds for the money-like values a work order and its lines carry.
//!
//! The forms enforce the same rules, but these commands are the single write
//! path to the database, so the rules are repeated here where nothing can skip them.

use rust_decimal::Decimal;

use crate::{
    database::work_orders::structs::{DiscountType, LaborCostType},
    error::{AppError, AppResult},
};

const MAX_AMOUNT: u64 = 10_000_000;
const MAX_LABOR_HOURS: u64 = 10_000;
const MAX_MARKUP: u64 = 300;
const MAX_ODOMETER: i32 = 10_000_000;

/// A price, cost or tariff: zero is allowed, negative and absurd values are not.
pub(crate) fn amount(value: Decimal, label: &str) -> AppResult<Decimal> {
    if value.is_sign_negative() {
        return Err(AppError::validation(format!(
            "{label} має бути невід'ємним числом"
        )));
    }
    if value > Decimal::from(MAX_AMOUNT) {
        return Err(AppError::validation(format!(
            "{label} не може перевищувати {MAX_AMOUNT}"
        )));
    }
    Ok(value)
}

/// Hours on a labour line. Unused by `Fixed` lines, but still bounded so a
/// stray value cannot skew the total of an hourly line later.
pub(crate) fn labor_hours(value: Decimal) -> AppResult<Decimal> {
    if value.is_sign_negative() {
        return Err(AppError::validation(
            "Кількість годин має бути невід'ємним числом",
        ));
    }
    if value > Decimal::from(MAX_LABOR_HOURS) {
        return Err(AppError::validation(format!(
            "Кількість годин не може перевищувати {MAX_LABOR_HOURS}"
        )));
    }
    Ok(value)
}

/// A part's markup percentage; the ceiling matches the form's 0–300 %.
pub(crate) fn markup(value: Decimal) -> AppResult<Decimal> {
    if value.is_sign_negative() || value > Decimal::from(MAX_MARKUP) {
        return Err(AppError::validation(format!(
            "Націнка має бути в межах від 0 до {MAX_MARKUP} %"
        )));
    }
    Ok(value)
}

/// Normalizes a line discount: a `None` discount never keeps a stray value in
/// the column, and a percentage can never exceed 100 %.
pub(crate) fn discount(kind: DiscountType, value: Decimal) -> AppResult<Decimal> {
    match kind {
        DiscountType::None => Ok(Decimal::ZERO),
        DiscountType::Percentage => {
            if value <= Decimal::ZERO || value > Decimal::ONE_HUNDRED {
                return Err(AppError::validation(
                    "Знижка у відсотках має бути в межах від 0 до 100 %",
                ));
            }
            Ok(value)
        }
        DiscountType::Fixed => {
            if value <= Decimal::ZERO {
                return Err(AppError::validation("Вкажіть суму знижки"));
            }
            amount(value, "Знижка")
        }
    }
}

/// The order's labour cost: an hourly figure or a percentage of the labour
/// revenue.
pub(crate) fn labor_cost(kind: LaborCostType, value: Decimal) -> AppResult<Decimal> {
    match kind {
        LaborCostType::Hourly => amount(value, "Вартість години робіт"),
        LaborCostType::Percentage => {
            if value.is_sign_negative() || value > Decimal::ONE_HUNDRED {
                return Err(AppError::validation(
                    "Відсоток вартості робіт має бути в межах від 0 до 100 %",
                ));
            }
            Ok(value)
        }
    }
}

/// Odometer readings are whole kilometres and cannot go backwards.
pub(crate) fn odometer(value: i32, label: &str) -> AppResult<i32> {
    if value < 0 {
        return Err(AppError::validation(format!(
            "{label} не може бути від'ємним"
        )));
    }
    if value > MAX_ODOMETER {
        return Err(AppError::validation(format!(
            "{label} не може перевищувати {MAX_ODOMETER} км"
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_rejects_negative_and_absurd_values() {
        assert!(amount(Decimal::from(-1), "Ціна").is_err());
        assert!(amount(Decimal::from(MAX_AMOUNT) + Decimal::ONE, "Ціна").is_err());
        assert!(amount(Decimal::ZERO, "Ціна").is_ok());
        assert!(amount(Decimal::from(500), "Ціна").is_ok());
    }

    #[test]
    fn percentage_discount_must_be_within_one_hundred() {
        assert!(discount(DiscountType::Percentage, Decimal::ZERO).is_err());
        assert!(discount(DiscountType::Percentage, Decimal::from(101)).is_err());
        assert!(discount(DiscountType::Percentage, Decimal::from(100)).is_ok());
        assert!(discount(DiscountType::Percentage, Decimal::from(15)).is_ok());
    }

    #[test]
    fn fixed_discount_must_be_positive() {
        assert!(discount(DiscountType::Fixed, Decimal::ZERO).is_err());
        assert!(discount(DiscountType::Fixed, Decimal::from(-10)).is_err());
        assert!(discount(DiscountType::Fixed, Decimal::from(10)).is_ok());
    }

    #[test]
    fn no_discount_drops_a_stray_value() {
        let value = discount(DiscountType::None, Decimal::from(50)).expect("none is always valid");
        assert!(value.is_zero());
    }

    #[test]
    fn markup_is_capped() {
        assert!(markup(Decimal::from(-1)).is_err());
        assert!(markup(Decimal::from(MAX_MARKUP + 1)).is_err());
        assert!(markup(Decimal::ZERO).is_ok());
        assert!(markup(Decimal::from(MAX_MARKUP)).is_ok());
    }

    #[test]
    fn labor_cost_percentage_is_capped() {
        assert!(labor_cost(LaborCostType::Percentage, Decimal::from(101)).is_err());
        assert!(labor_cost(LaborCostType::Percentage, Decimal::from(100)).is_ok());
        assert!(labor_cost(LaborCostType::Hourly, Decimal::from(500)).is_ok());
        assert!(labor_cost(LaborCostType::Hourly, Decimal::from(-5)).is_err());
    }

    #[test]
    fn odometer_rejects_negative_readings() {
        assert!(odometer(-1, "Пробіг").is_err());
        assert!(odometer(0, "Пробіг").is_ok());
        assert!(odometer(250_000, "Пробіг").is_ok());
    }
}
