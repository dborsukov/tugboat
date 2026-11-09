import { describe, expect, it } from "vitest";

import { zBigString } from "./zbig";

const firstIssue = (schema: ReturnType<typeof zBigString>, value: string) => {
  const result = schema.safeParse(value);
  return result.success ? undefined : result.error.issues[0]?.message;
};

describe("zBigString", () => {
  it("normalizes a comma used as the decimal separator", () => {
    expect(zBigString().parse("1,5")).toBe("1.5");
  });

  it("trims surrounding whitespace", () => {
    expect(zBigString().parse("  42  ")).toBe("42");
  });

  it("rejects an empty value with the required message", () => {
    expect(firstIssue(zBigString({ requiredError: "Вкажіть суму" }), "")).toBe(
      "Вкажіть суму",
    );
  });

  it("rejects a value that is not a number", () => {
    expect(
      firstIssue(zBigString({ invalidError: "Некоректний формат" }), "abc"),
    ).toBe("Некоректний формат");
  });

  it("rejects whitespace that collapses to an empty number", () => {
    expect(zBigString().safeParse("   ").success).toBe(false);
  });

  it("enforces the minimum", () => {
    const schema = zBigString({ min: { value: 0, message: "Не менше нуля" } });

    expect(firstIssue(schema, "-1")).toBe("Не менше нуля");
    expect(schema.safeParse("0").success).toBe(true);
  });

  it("enforces the maximum", () => {
    const schema = zBigString({
      max: { value: 100, message: "Не більше 100" },
    });

    expect(firstIssue(schema, "100.01")).toBe("Не більше 100");
    expect(schema.safeParse("100").success).toBe(true);
  });

  it("compares bounds numerically, not lexicographically", () => {
    const schema = zBigString({ max: { value: 9, message: "Занадто велике" } });

    // A string comparison would treat "10" as smaller than "9".
    expect(firstIssue(schema, "10")).toBe("Занадто велике");
  });
});
