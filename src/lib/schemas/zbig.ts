import Big from "big.js";
import z from "zod";

interface BigStringOptions {
  requiredError?: string;
  invalidError?: string;
  min?: { value: number | string; message: string };
  max?: { value: number | string; message: string };
}

export const zBigString = (opts: BigStringOptions = {}) => {
  const {
    requiredError = "Обов'язкове поле",
    invalidError = "Некоректний формат числа",
    min,
    max,
  } = opts;

  return z
    .string()
    .min(1, { message: requiredError, abort: true })
    .superRefine((val, ctx) => {
      let b: Big;

      // Catch parsing/instantiation failures
      try {
        b = new Big(val);
      } catch {
        ctx.addIssue({
          code: "custom",
          message: invalidError,
        });
        return;
      }

      // Check minimum boundary
      if (min !== undefined && b.lt(min.value)) {
        ctx.addIssue({
          code: "custom",
          message: min.message,
        });
      }

      // Check maximum boundary
      if (max !== undefined && b.gt(max.value)) {
        ctx.addIssue({
          code: "custom",
          message: max.message,
        });
      }
    });
};
