import { describe, expect, it } from "vitest";
import z from "zod";

import {
  discountFields,
  discountFromEntity,
  discountToPayload,
  refineDiscount,
} from "./zdiscount";

const schema = z.object(discountFields).superRefine(refineDiscount);

const firstIssue = (input: Record<string, string>) => {
  const result = schema.safeParse(input);
  return result.success ? undefined : result.error.issues[0]?.message;
};

describe("discount schema", () => {
  it("accepts a missing discount and normalizes its value to zero", () => {
    expect(schema.parse({ discountType: "None" }).discountValue).toBe("0");
  });

  it("rejects a percentage of zero", () => {
    expect(firstIssue({ discountType: "Percentage", discountValue: "0" })).toBe(
      "Вкажіть знижку",
    );
  });

  it("rejects a percentage above one hundred", () => {
    expect(
      firstIssue({ discountType: "Percentage", discountValue: "101" }),
    ).toBe("Знижка у відсотках має бути в межах 100%");
  });

  it("accepts the percentage boundaries", () => {
    expect(
      schema.safeParse({ discountType: "Percentage", discountValue: "1" })
        .success,
    ).toBe(true);
    expect(
      schema.safeParse({ discountType: "Percentage", discountValue: "100" })
        .success,
    ).toBe(true);
  });

  it("rejects a fixed discount that is not positive", () => {
    expect(firstIssue({ discountType: "Fixed", discountValue: "0" })).toBe(
      "Вкажіть знижку",
    );
    expect(firstIssue({ discountType: "Fixed", discountValue: "-5" })).toBe(
      "Вкажіть знижку",
    );
  });

  it("accepts a positive fixed discount", () => {
    expect(
      schema.safeParse({ discountType: "Fixed", discountValue: "250" }).success,
    ).toBe(true);
  });
});

describe("discount payload mapping", () => {
  it("stores zero when the discount type is none", () => {
    expect(discountToPayload("None", "50")).toEqual({
      discountType: "None",
      discountValue: "0",
    });
  });

  it("keeps the value for the other types", () => {
    expect(discountToPayload("Percentage", "15")).toEqual({
      discountType: "Percentage",
      discountValue: "15",
    });
    expect(discountToPayload("Fixed", "250")).toEqual({
      discountType: "Fixed",
      discountValue: "250",
    });
  });

  it("reads zero back from a none row", () => {
    expect(
      discountFromEntity({ discountType: "None", discountValue: "50" }),
    ).toBe("0");
    expect(
      discountFromEntity({ discountType: "Fixed", discountValue: "250" }),
    ).toBe("250");
  });
});
