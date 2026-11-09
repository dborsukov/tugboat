import { describe, expect, it } from "vitest";
import z from "zod";

import {
  laborCostFields,
  laborCostToPayload,
  refineLaborCost,
} from "./zlaborcost";

const schema = z.object(laborCostFields).superRefine(refineLaborCost);

const firstIssue = (input: Record<string, string>) => {
  const result = schema.safeParse(input);
  return result.success ? undefined : result.error.issues[0]?.message;
};

describe("labor cost schema", () => {
  it("keeps a percentage within zero and one hundred", () => {
    expect(
      schema.safeParse({ laborCostType: "Percentage", laborCost: "100" })
        .success,
    ).toBe(true);
    expect(firstIssue({ laborCostType: "Percentage", laborCost: "101" })).toBe(
      "Некоректний відсоток",
    );
    expect(firstIssue({ laborCostType: "Percentage", laborCost: "-1" })).toBe(
      "Некоректний відсоток",
    );
  });

  it("rejects a negative hourly cost", () => {
    expect(firstIssue({ laborCostType: "Hourly", laborCost: "-0.01" })).toBe(
      "Некоректне число",
    );
    expect(
      schema.safeParse({ laborCostType: "Hourly", laborCost: "500" }).success,
    ).toBe(true);
  });
});

describe("labor cost payload mapping", () => {
  it("passes the value through with the chosen type", () => {
    expect(laborCostToPayload("Percentage", "30")).toEqual({
      laborCostType: "Percentage",
      laborCostValue: "30",
    });
    expect(laborCostToPayload("Hourly", "450")).toEqual({
      laborCostType: "Hourly",
      laborCostValue: "450",
    });
  });
});
