import Big from "big.js";
import z, { type RefinementCtx } from "zod";

import type {
  CreateExpensePayload,
  Expense,
  PaymentMethod,
  VatScheme,
} from "$lib/bindings";

import { zBigString } from "./zbig";

const dayPattern = /^\d{4}-\d{2}-\d{2}$/;

export const expenseFields = {
  date: z
    .string()
    .regex(dayPattern, { error: "Вкажіть дату витрати" })
    .default(""),
  amount: zBigString({
    requiredError: "Вкажіть суму витрати",
    min: { value: 0, message: "Сума не може бути від'ємною" },
  }).default(""),
  description: z
    .string()
    .trim()
    .min(1, { error: "Вкажіть опис витрати" })
    .max(300, { error: "Опис не може бути довшим за 300 символів" })
    .default(""),
  categoryId: z.number().int().positive().nullable().default(null),
  paymentMethod: z
    .enum<PaymentMethod[]>(["Cash", "Card", "BankTransfer", "CardTransfer"])
    .nullable()
    .default(null),
  vatRate: zBigString({
    requiredError: "Вкажіть ставку ПДВ",
    min: { value: 0, message: "Ставка ПДВ має бути від 0 до 100" },
    max: { value: 100, message: "Ставка ПДВ має бути від 0 до 100" },
  }).default("0"),
  priceIncludesVat: z.boolean().default(true),
  vatScheme: z
    .enum<VatScheme[]>(["standard", "zero", "exempt", "not_object", "export"])
    .default("standard"),
  supplierName: z
    .string()
    .trim()
    .max(200, {
      error: "Назва постачальника не може бути довшою за 200 символів",
    })
    .default(""),
  supplierCode: z
    .string()
    .trim()
    .max(20, {
      error: "Код постачальника не може бути довшим за 20 символів",
    })
    .default(""),
  invoiceNumber: z
    .string()
    .trim()
    .max(60, {
      error: "Номер податкової накладної не може бути довшим за 60 символів",
    })
    .default(""),
  invoiceDate: z
    .union([
      z.literal(""),
      z
        .string()
        .regex(dayPattern, { error: "Вкажіть дату податкової накладної" }),
    ])
    .default(""),
  vatCredited: z.boolean().default(false),
};

export interface ExpenseFormShape {
  amount?: string;
  vatRate?: string;
  vatScheme?: VatScheme;
  supplierName?: string;
  invoiceNumber?: string;
  invoiceDate?: string;
  vatCredited?: boolean;
}

export const refineExpense = (data: ExpenseFormShape, ctx: RefinementCtx) => {
  if (data.amount === "") {
    // handled by zBigString
  } else {
    let amount: Big | null = null;
    try {
      amount = new Big(data.amount!);
    } catch {
      // handled by zBigString
    }
    if (amount?.lte(0)) {
      ctx.addIssue({
        code: "custom",
        message: "Сума має бути більшою за нуль",
        path: ["amount"],
      });
    }
  }

  if (!data.vatCredited) return;

  // A credit has to be backed by a supplier tax invoice, or it cannot be
  // claimed and would not survive a review.
  let rate: Big | null = null;
  try {
    rate = new Big(data.vatRate || "0");
  } catch {
    // handled by zBigString
  }
  if (rate != null && rate.lte(0)) {
    ctx.addIssue({
      code: "custom",
      message: "Для податкового кредиту вкажіть ставку ПДВ",
      path: ["vatRate"],
    });
  }
  if (data.vatScheme && data.vatScheme !== "standard" && rate?.gt(0)) {
    ctx.addIssue({
      code: "custom",
      message: "Для пільгової ставки вкажіть 0 %",
      path: ["vatRate"],
    });
  }
  if (!data.supplierName?.trim()) {
    ctx.addIssue({
      code: "custom",
      message: "Вкажіть постачальника для податкового кредиту",
      path: ["supplierName"],
    });
  }
  if (!data.invoiceNumber?.trim()) {
    ctx.addIssue({
      code: "custom",
      message: "Вкажіть номер податкової накладної",
      path: ["invoiceNumber"],
    });
  }
  if (!data.invoiceDate) {
    ctx.addIssue({
      code: "custom",
      message: "Вкажіть дату податкової накладної",
      path: ["invoiceDate"],
    });
  }
};

function decimalOf(value: string): Big {
  return new Big((value || "0").replace(/\s/g, "").replace(",", "."));
}

export function expenseVatAmount(
  amount: string,
  vatRate: string,
  priceIncludesVat: boolean,
): Big {
  try {
    const total = decimalOf(amount);
    const rate = decimalOf(vatRate);
    if (total.lte(0) || rate.lte(0)) return new Big(0);

    const hundred = new Big(100);
    return priceIncludesVat
      ? total.times(rate).div(hundred.plus(rate))
      : total.times(rate).div(hundred);
  } catch {
    return new Big(0);
  }
}

export function expenseVatBase(
  amount: string,
  vatRate: string,
  priceIncludesVat: boolean,
): Big {
  try {
    const total = decimalOf(amount);
    return priceIncludesVat
      ? total.minus(expenseVatAmount(amount, vatRate, priceIncludesVat))
      : total;
  } catch {
    return new Big(0);
  }
}

export function expenseToPayload(data: {
  date: string;
  amount: string;
  description: string;
  categoryId: number | null;
  paymentMethod: PaymentMethod | null;
  vatRate: string;
  priceIncludesVat: boolean;
  vatScheme: VatScheme;
  supplierName: string;
  supplierCode: string;
  invoiceNumber: string;
  invoiceDate: string;
  vatCredited: boolean;
}): CreateExpensePayload {
  return {
    date: data.date,
    amount: data.amount.trim().replace(",", "."),
    description: data.description.trim(),
    categoryId: data.categoryId,
    paymentMethod: data.paymentMethod,
    vatRate: data.vatRate.trim().replace(",", "."),
    priceIncludesVat: data.priceIncludesVat,
    vatScheme: data.vatScheme,
    supplierName: data.supplierName.trim() || null,
    supplierCode: data.supplierCode.trim() || null,
    invoiceNumber: data.invoiceNumber.trim() || null,
    invoiceDate: data.invoiceDate || null,
    vatCredited: data.vatCredited,
  };
}

export function expenseToForm(expense: Expense) {
  return {
    date: expense.date,
    amount: expense.amount,
    description: expense.description,
    categoryId: expense.category?.id ?? null,
    paymentMethod: expense.paymentMethod,
    vatRate: expense.vatRate,
    priceIncludesVat: expense.priceIncludesVat,
    vatScheme: expense.vatScheme,
    supplierName: expense.supplierName ?? "",
    supplierCode: expense.supplierCode ?? "",
    invoiceNumber: expense.invoiceNumber ?? "",
    invoiceDate: expense.invoiceDate ?? "",
    vatCredited: expense.vatCredited,
  };
}
