import { describe, expect, it } from "vitest";
import z from "zod";

import {
  laborPriceFields,
  laborPriceToPayload,
  refineLaborPrice,
} from "./zlaborprice";

const schema = z.object(laborPriceFields).superRefine(refineLaborPrice);

const firstIssue = (input: Record<string, string>) => {
  const result = schema.safeParse(input);
  return result.success ? undefined : result.error.issues[0]?.message;
};

describe("labor price schema", () => {
  it("rejects a price of zero or less", () => {
    expect(firstIssue({ laborPriceType: "Hourly", laborPrice: "0" })).toBe(
      "Некоректне число",
    );
    expect(firstIssue({ laborPriceType: "Fixed", laborPrice: "-10" })).toBe(
      "Некоректне число",
    );
  });

  it("accepts a positive price on both types", () => {
    expect(
      schema.safeParse({ laborPriceType: "Hourly", laborPrice: "0.01" })
        .success,
    ).toBe(true);
    expect(
      schema.safeParse({ laborPriceType: "Fixed", laborPrice: "900" }).success,
    ).toBe(true);
  });
});

describe("labor price payload mapping", () => {
  it("puts an hourly price into the hours column", () => {
    expect(laborPriceToPayload("Hourly", "500")).toEqual({
      priceType: "Hourly",
      fixedPrice: "0",
      laborHours: "500",
    });
  });

  it("puts a fixed price into the price column", () => {
    expect(laborPriceToPayload("Fixed", "500")).toEqual({
      priceType: "Fixed",
      fixedPrice: "500",
      laborHours: "0",
    });
  });
});
