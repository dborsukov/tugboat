//! The figures a VAT register prints.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use super::{figures::ReportContext, structs::DayRange};
use crate::database::vat::structs::{VatDirection, VatEntry, VatScheme};

const MONTHS: [&str; 12] = [
    "Січень",
    "Лютий",
    "Березень",
    "Квітень",
    "Травень",
    "Червень",
    "Липень",
    "Серпень",
    "Вересень",
    "Жовтень",
    "Листопад",
    "Грудень",
];

/// One row of the register.
#[derive(Debug, Clone)]
pub(crate) struct VatEntryFigure {
    pub direction: VatDirection,
    /// `None` when the stored text does not parse as a date; the text is still
    /// printed as it stands rather than moved to the start of the period.
    pub tax_point: Option<NaiveDate>,
    pub tax_point_text: String,
    pub document: String,
    /// The order id or the expense id the entry was made from.
    pub source_ref: String,
    pub counterparty: String,
    pub code: String,
    pub base: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    /// How the entry is treated: standard, zero, exempt, not-object or export.
    pub scheme: VatScheme,
    /// The benefit code, for an exempt supply.
    pub exemption_code: Option<String>,
    /// True on the row that cancels an earlier entry.
    pub reversal: bool,
}

impl VatEntryFigure {
    /// What the row is, as the register names it.
    pub fn kind(&self) -> &'static str {
        if self.reversal {
            "Сторно"
        } else {
            match self.direction {
                VatDirection::Output => "Нараховано",
                VatDirection::Input => "Кредит",
            }
        }
    }

    /// The treatment, when it is not the ordinary one.
    pub fn scheme_label(&self) -> &'static str {
        if self.scheme.is_standard() {
            ""
        } else {
            self.scheme.label()
        }
    }
}

/// One rate's share of the period, both sides at once.
#[derive(Debug, Clone)]
pub(crate) struct VatRateFigure {
    pub rate: Decimal,
    pub scheme: VatScheme,
    pub output_base: Decimal,
    pub output_amount: Decimal,
    pub input_base: Decimal,
    pub input_amount: Decimal,
}

/// One month's share of the period, both sides at once.
#[derive(Debug, Clone)]
pub(crate) struct VatMonthFigure {
    pub label: String,
    pub output_amount: Decimal,
    pub input_amount: Decimal,
}

impl VatMonthFigure {
    pub fn net(&self) -> Decimal {
        self.output_amount - self.input_amount
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct VatTotals {
    pub base: Decimal,
    pub amount: Decimal,
    /// Rows that add to the period, as opposed to reversals of earlier rows.
    pub entries: usize,
    pub reversals: usize,
}

/// A VAT register's whole set of figures, ready to be drawn.
pub(crate) struct VatReportFigures {
    pub period: DayRange,
    pub company: String,
    pub subtitle: String,
    pub output: VatTotals,
    pub input: VatTotals,
    /// Output VAT less input VAT: what the period owes, or would be refunded.
    pub net: Decimal,
    pub net_base: Decimal,
    pub by_rate: Vec<VatRateFigure>,
    pub by_month: Vec<VatMonthFigure>,
    pub entries: Vec<VatEntryFigure>,
}

impl VatReportFigures {
    pub fn new(context: &ReportContext, period: DayRange, entries: &[VatEntry]) -> Self {
        let mut output = VatTotals::default();
        let mut input = VatTotals::default();
        let mut rows = Vec::with_capacity(entries.len());

        // Per rate and treatment, and per month: output base, output VAT,
        // input base, input VAT.
        let mut by_rate: BTreeMap<(Decimal, VatScheme), [Decimal; 4]> = BTreeMap::new();
        let mut by_month: BTreeMap<(i32, u32), [Decimal; 2]> = BTreeMap::new();

        for entry in entries {
            let totals = match entry.direction {
                VatDirection::Output => &mut output,
                VatDirection::Input => &mut input,
            };
            totals.base += entry.base;
            totals.amount += entry.amount;
            if entry.reverses_id.is_some() {
                totals.reversals += 1;
            } else {
                totals.entries += 1;
            }
            let parsed = NaiveDate::parse_from_str(&entry.tax_point, "%Y-%m-%d").ok();
            let tax_point = parsed.unwrap_or(period.start);

            let offset = match entry.direction {
                VatDirection::Output => 0,
                VatDirection::Input => 2,
            };
            let rate_slot = by_rate
                .entry((entry.rate, entry.scheme))
                .or_insert([Decimal::ZERO; 4]);
            rate_slot[offset] += entry.base;
            rate_slot[offset + 1] += entry.amount;

            // The month table shows what each side charged, not its base.
            let month_index = match entry.direction {
                VatDirection::Output => 0,
                VatDirection::Input => 1,
            };
            let month_slot = by_month
                .entry((tax_point.year(), tax_point.month()))
                .or_insert([Decimal::ZERO; 2]);
            month_slot[month_index] += entry.amount;

            rows.push(VatEntryFigure {
                direction: entry.direction,
                tax_point: parsed,
                tax_point_text: entry.tax_point.clone(),
                document: entry
                    .document_number
                    .clone()
                    .unwrap_or_else(|| entry.source_id.clone()),
                source_ref: entry.source_id.clone(),
                counterparty: entry.counterparty_name.clone(),
                code: entry.counterparty_code.clone().unwrap_or_default(),
                base: entry.base,
                rate: entry.rate,
                amount: entry.amount,
                scheme: entry.scheme,
                exemption_code: entry.exemption_code.clone(),
                reversal: entry.reverses_id.is_some(),
            });
        }

        // Highest rate first, the way a return lists them.
        let mut by_rate: Vec<VatRateFigure> = by_rate
            .into_iter()
            .map(|((rate, scheme), sums)| VatRateFigure {
                rate,
                scheme,
                output_base: sums[0],
                output_amount: sums[1],
                input_base: sums[2],
                input_amount: sums[3],
            })
            .collect();
        by_rate.sort_by_key(|row| (std::cmp::Reverse(row.rate), row.scheme));

        let by_month: Vec<VatMonthFigure> = by_month
            .into_iter()
            .map(|((year, month), sums)| VatMonthFigure {
                label: format!(
                    "{} {}",
                    MONTHS[(month.saturating_sub(1) as usize).min(11)],
                    year
                ),
                output_amount: sums[0],
                input_amount: sums[1],
            })
            .collect();

        let net = output.amount - input.amount;
        let net_base = output.base - input.base;

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
            output,
            input,
            net,
            net_base,
            by_rate,
            by_month,
            entries: rows,
        }
    }
}
