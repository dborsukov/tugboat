import Big from "big.js";
import z, { type RefinementCtx } from "zod";

import type { LaborCostType } from "$lib/bindings";

import { zBigString } from "./zbig";

export const laborCostFields = {
  laborCostType: z
    .enum<LaborCostType[]>(["Hourly", "Percentage"])
    .default("Percentage"),
  laborCost: zBigString({
    requiredError: "Обов'язкове поле",
  }).default("0"),
};

export interface LaborCostFormShape {
  laborCostType?: LaborCostType;
  laborCost?: string;
}

export const refineLaborCost = (
  data: LaborCostFormShape,
  ctx: RefinementCtx,
) => {
  if (data.laborCost === "") return; // handled by zBigString

  let laborCostBig: Big;
  try {
    laborCostBig = new Big(data.laborCost!);
  } catch {
    return; // handled by zBigString
  }

  if (data.laborCostType === "Percentage") {
    if (laborCostBig.lt(0)) {
      ctx.addIssue({
        code: "custom",
        message: "Некоректний відсоток",
        path: ["laborCost"],
      });
      return;
    }
    if (laborCostBig.gt(100)) {
      ctx.addIssue({
        code: "custom",
        message: "Некоректний відсоток",
        path: ["laborCost"],
      });
      return;
    }
  } else if (data.laborCostType === "Hourly") {
    if (laborCostBig.lt(0)) {
      ctx.addIssue({
        code: "custom",
        message: "Некоректне число",
        path: ["discount"],
      });
      return;
    }
  }
};

export function laborCostToPayload(
  laborCostType: LaborCostType,
  laborCost: string,
) {
  return {
    laborCostType: laborCostType,
    laborCostValue: laborCost,
  };
}

export function laborCostFromEntity(entity: {
  laborCostType: LaborCostType;
  laborCostValue: string;
}) {
  return entity.laborCostValue;
}
