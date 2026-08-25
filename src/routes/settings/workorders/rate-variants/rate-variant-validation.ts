import Big from "big.js";
import z from "zod";

import { fieldError } from "$lib/zod";

const RATE_VARIANT_NAME_MAX_LENGTH = 60;
/** matches "0", "0.5", "120.00" - a non-negative number with up to 2 decimals */
const DECIMAL_PATTERN = /^\d+(\.\d{1,2})?$/;

const rateVariantNameSchema = z
  .string()
  .trim()
  .min(1, "Вкажіть назву")
  .max(RATE_VARIANT_NAME_MAX_LENGTH, "Занадто довга назва");

const rateVariantValueSchema = z
  .string()
  .trim()
  .transform((v) => v.replace(",", "."))
  .refine((v) => v.length > 0, "Вкажіть суму")
  .refine(
    (v) => DECIMAL_PATTERN.test(v),
    "Додатне число, до 2 знаків після коми",
  );

/** Normalize an already-validated raw value into a canonical "123.45" string. */
export function normalizeRateVariantValue(raw: string): string {
  const parsed = rateVariantValueSchema.safeParse(raw);
  const value = parsed.success ? parsed.data : raw.trim().replace(",", ".");
  return new Big(value).toFixed(2);
}

/**
 * Validates a single entry (as typed in the create/edit dialog) against the
 * rest of the list, so duplicate names are caught across rows.
 */
export function validateRateVariantEntry(
  entry: { name: string; value: string },
  others: { name: string; value: string }[],
): { nameError?: string; valueError?: string } {
  const nameResult = rateVariantNameSchema.safeParse(entry.name);
  let nameError: string | undefined;
  if (!nameResult.success) {
    nameError = nameResult.error.issues[0]?.message;
  } else if (
    others.some(
      (o) => o.name.trim().toLowerCase() === nameResult.data.toLowerCase(),
    )
  ) {
    nameError = "Така назва вже використовується";
  }
  return {
    nameError,
    valueError: fieldError(rateVariantValueSchema, entry.value),
  };
}
