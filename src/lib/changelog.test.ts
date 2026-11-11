import { describe, expect, it } from "vitest";

import { parseChangelog, plainChangelog } from "./changelog";

describe("parseChangelog", () => {
  it("groups bullets under their category", () => {
    const body = [
      "## Нове",
      "- Експорт звітів у PDF",
      "## Виправлено",
      "- Маржа у звітах",
    ].join("\n");

    const sections = parseChangelog(body);

    expect(sections).toHaveLength(2);
    expect(sections[0]).toMatchObject({
      kind: "new",
      items: ["Експорт звітів у PDF"],
    });
    expect(sections[1]).toMatchObject({
      kind: "fixed",
      items: ["Маржа у звітах"],
    });
  });

  it("accepts English headings and different heading levels", () => {
    const sections = parseChangelog("### Improved\n- Faster search\n");

    expect(sections).toHaveLength(1);
    expect(sections[0].kind).toBe("improved");
  });

  it("ignores categories it does not know", () => {
    const sections = parseChangelog(
      "## Відомо\n- Не зламано\n## Нове\n- Щось нове\n",
    );

    expect(sections).toHaveLength(1);
    expect(sections[0].kind).toBe("new");
  });

  it("joins wrapped bullets", () => {
    const sections = parseChangelog(
      "## Нове\n- Перший рядок\n  продовження опису\n",
    );

    expect(sections[0].items).toEqual(["Перший рядок продовження опису"]);
  });

  it("returns nothing when the notes have no categories", () => {
    expect(parseChangelog("Просто текст\nз двох рядків")).toEqual([]);
    expect(parseChangelog(null)).toEqual([]);
  });
});

describe("plainChangelog", () => {
  it("drops headings and blank lines", () => {
    expect(plainChangelog("# Заголовок\n\nПерший абзац\n\nДругий")).toEqual([
      "Перший абзац",
      "Другий",
    ]);
  });

  it("returns nothing without a body", () => {
    expect(plainChangelog(undefined)).toEqual([]);
  });
});
