import type { PaymentMethod } from "$lib/bindings";

/** Method names on their own, e.g. inside a picker. */
export const paymentMethodLabels: Record<PaymentMethod, string> = {
  Cash: "Готівка",
  Card: "Картка",
  BankTransfer: "Безготівковий (рахунок)",
  CardTransfer: "Переказ на картку",
};

/**
 * Instrumental-case names, for places where the method is part of a sentence:
 * "Сплачено 10 вересня 2026 р. о 17:12 · Карткою".
 */
export const paymentMethodInstrumentalLabels: Record<PaymentMethod, string> = {
  Cash: "Готівкою",
  Card: "Карткою",
  BankTransfer: "Безготівковим розрахунком",
  CardTransfer: "Переказом на картку",
};

/** Instrumental-case name of a method, or an empty string when there is none. */
export function paymentMethodInstrumentalLabel(
  method: PaymentMethod | null | undefined,
): string {
  return method ? paymentMethodInstrumentalLabels[method] : "";
}
