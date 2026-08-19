import Big from "big.js";
import z, { type RefinementCtx } from "zod";

import type { DiscountType } from "$lib/bindings";

import { zBigString } from "./zbig";

export const discountFields = {
  discountType: z
    .enum<DiscountType[]>(["None", "Fixed", "Percentage"])
    .default("None"),
  discountValue: zBigString({
    requiredError: "Вкажіть знижку",
  }).default("0"),
};

export interface DiscountFormShape {
  discountType?: DiscountType;
  discountValue?: string;
}

export const refineDiscount = (data: DiscountFormShape, ctx: RefinementCtx) => {
  if (data.discountType === "None") return;
  if (data.discountValue === "") return; // handled by zBigString

  let discountBig: Big;
  try {
    discountBig = new Big(data.discountValue!);
  } catch {
    return; // handled by zBigString
  }

  if (data.discountType === "Percentage") {
    if (discountBig.lte(0)) {
      ctx.addIssue({
        code: "custom",
        message: "Вкажіть знижку",
        path: ["discount"],
      });
      return;
    }
    if (discountBig.gt(100)) {
      ctx.addIssue({
        code: "custom",
        message: "Знижка у відсотках має бути в межах 100%",
        path: ["discount"],
      });
      return;
    }
  } else if (data.discountType === "Fixed") {
    if (discountBig.lte(0)) {
      ctx.addIssue({
        code: "custom",
        message: "Вкажіть знижку",
        path: ["discount"],
      });
      return;
    }
    if (discountBig.lt(0)) {
      ctx.addIssue({
        code: "custom",
        message: "Знижка не може бути від'ємною",
        path: ["discount"],
      });
      return;
    }
  }
};

export function discountToPayload(
  discountType: DiscountType,
  discountValue: string,
) {
  if (discountType === "None") {
    return {
      discountType,
      discountValue: "0",
    };
  } else {
    return {
      discountType,
      discountValue: discountValue,
    };
  }
}

export function discountFromEntity(entity: {
  discountType: DiscountType;
  discountValue: string;
}) {
  if (entity.discountType === "None") {
    return "0";
  } else {
    return entity.discountValue;
  }
}
