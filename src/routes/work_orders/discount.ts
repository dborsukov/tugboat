import z from "zod";

import type { DiscountType } from "$lib/bindings";

export const discountFields = {
  discountType: z.enum(["Fixed", "Percentage"]).default("Fixed"),
  discount: z.number({ error: "Вкажіть знижку" }).default(0),
};

export function refineDiscount(
  data: { discountType: DiscountType; discount: number },
  ctx: z.RefinementCtx,
) {
  if (data.discountType === "Fixed") {
    if (data.discount < 0) {
      ctx.addIssue({
        code: "custom",
        path: ["discount"],
        message: "Знижка не може бути від'ємною",
      });
    }
  } else {
    if (data.discount < 0) {
      ctx.addIssue({
        code: "custom",
        path: ["discount"],
        message: "Відсоток не може бути від'ємним",
      });
    }
    if (data.discount > 100) {
      ctx.addIssue({
        code: "custom",
        path: ["discount"],
        message: "Відсоток не може перевищувати 100",
      });
    }
  }
}

export function discountToPayload(
  discountType: DiscountType,
  discount: number,
) {
  return discountType === "Fixed"
    ? {
        discountType,
        discountCents: Math.round(discount * 100),
        discountPercent: 0,
      }
    : { discountType, discountCents: 0, discountPercent: discount };
}

export function discountFromEntity(entity: {
  discountType: DiscountType;
  discountCents: number;
  discountPercent: number;
}) {
  return entity.discountType === "Fixed"
    ? entity.discountCents / 100
    : entity.discountPercent;
}
