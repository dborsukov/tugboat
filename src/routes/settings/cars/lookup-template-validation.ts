import z from "zod";

import { fieldError } from "$lib/zod";

const LABEL_MAX_LENGTH = 60;
const URL_MAX_LENGTH = 500;
/** Placeholder inside the url template that gets replaced with the real code. */
const LOOKUP_PLACEHOLDER = "%s";

const labelSchema = z
  .string()
  .trim()
  .min(1, "Вкажіть назву")
  .max(LABEL_MAX_LENGTH, "Занадто довга назва");

function isValidHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

const urlTemplateSchema = z
  .string()
  .trim()
  .min(1, "Вкажіть URL")
  .max(URL_MAX_LENGTH, "Занадто довгий URL")
  .refine(
    (v) => v.includes(LOOKUP_PLACEHOLDER),
    `URL має містити плейсхолдер ${LOOKUP_PLACEHOLDER}`,
  )
  .refine(isValidHttpUrl, "URL має починатися з http:// або https://");

export type LookupTemplateEntry = { label: string; urlTemplate: string };

/** Normalize an entry into its canonical stored form (trimmed). */
export function normalizeLookupTemplate(
  entry: LookupTemplateEntry,
): LookupTemplateEntry {
  return {
    label: entry.label.trim(),
    urlTemplate: entry.urlTemplate.trim(),
  };
}

/**
 * Validates a single entry (as typed in the create/edit dialog) against the
 * rest of the list, so duplicate labels are caught across rows.
 */
export function validateLookupTemplateEntry(
  entry: LookupTemplateEntry,
  others: LookupTemplateEntry[],
): { labelError?: string; urlTemplateError?: string } {
  const labelResult = labelSchema.safeParse(entry.label);
  let labelError: string | undefined;
  if (!labelResult.success) {
    labelError = labelResult.error.issues[0]?.message;
  } else if (
    others.some(
      (o) => o.label.trim().toLowerCase() === labelResult.data.toLowerCase(),
    )
  ) {
    labelError = "Така назва вже використовується";
  }
  return {
    labelError,
    urlTemplateError: fieldError(urlTemplateSchema, entry.urlTemplate),
  };
}
