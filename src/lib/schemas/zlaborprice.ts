import Big from "big.js";
import z, { type RefinementCtx } from "zod";

import type { LaborPriceType } from "$lib/bindings";

import { zBigString } from "./zbig";

export const laborPriceFields = {
  laborPriceType: z
    .enum<LaborPriceType[]>(["Hourly", "Fixed"])
    .default("Hourly"),
  laborPrice: zBigString({
    requiredError: "Обов'язкове поле",
  }).default("0"),
};

export interface LaborPriceFormShape {
  laborPriceType?: LaborPriceType;
  laborPrice?: string;
}

export const refineLaborPrice = (
  data: LaborPriceFormShape,
  ctx: RefinementCtx,
) => {
  if (data.laborPrice === "") return; // handled by zBigString

  let laborPriceBig: Big;
  try {
    laborPriceBig = new Big(data.laborPrice!);
  } catch {
    return; // handled by zBigString
  }

  if (laborPriceBig.lte(0)) {
    ctx.addIssue({
      code: "custom",
      message: "Некоректне число",
      path: ["laborPrice"],
    });
    return;
  }
};

export function laborPriceToPayload(
  laborPriceType: LaborPriceType,
  laborPrice: string,
) {
  if (laborPriceType === "Fixed") {
    return {
      priceType: laborPriceType,
      fixedPrice: laborPrice,
      laborHours: "0",
    };
  } else {
    return {
      priceType: laborPriceType,
      fixedPrice: "0",
      laborHours: laborPrice,
    };
  }
}

export function laborPriceFromEntity(entity: {
  priceType: LaborPriceType;
  fixedPrice: string;
  laborHours: string;
}) {
  return entity.priceType === "Fixed" ? entity.fixedPrice : entity.laborHours;
}
