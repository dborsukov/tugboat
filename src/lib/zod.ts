import type { ZodTypeAny } from "zod";

/** Runs a schema against a value and returns just the first error message, if any. */
export function fieldError(
  schema: ZodTypeAny,
  value: unknown,
): string | undefined {
  const result = schema.safeParse(value);
  return result.success ? undefined : result.error.issues[0]?.message;
}
