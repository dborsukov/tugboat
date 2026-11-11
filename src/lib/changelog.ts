/**
 * Release notes as the update dialog shows them.
 *
 * The updater only carries the release body as plain text, so the notes follow
 * a small convention - a markdown file with heading per category and bullet per change.
 *
 * Anything that does not match a known heading stays out of the structured
 * list and the dialog falls back to plain paragraphs when nothing matches.
 */

export type ChangelogKind = "new" | "improved" | "fixed";

export type ChangelogSection = {
  kind: ChangelogKind;
  title: string;
  items: string[];
};

const HEADINGS: readonly (readonly [RegExp, ChangelogKind])[] = [
  [/^(нове|нові|new)(?:\s|$)/i, "new"],
  [/^(покращено|покращення|improved|enhanced)(?:\s|$)/i, "improved"],
  [/^(виправлено|виправлення|fixed|fixes)(?:\s|$)/i, "fixed"],
];

/**
 * Groups the notes into the categories the dialog badges.
 * Returns an empty list when the body uses a different or broken format.
 */
export function parseChangelog(
  body: string | null | undefined,
): ChangelogSection[] {
  if (!body) return [];

  const sections: ChangelogSection[] = [];
  let current: ChangelogSection | null = null;

  for (const raw of body.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;

    const heading = /^#{1,4}\s+(.*)$/.exec(line);
    if (heading) {
      const title = heading[1].trim();
      const match = HEADINGS.find(([pattern]) => pattern.test(title));
      current = match ? { kind: match[1], title, items: [] } : null;
      if (current) sections.push(current);
      continue;
    }

    const item = /^[-*]\s+(.*)$/.exec(line);
    if (item && current) {
      current.items.push(item[1].trim());
      continue;
    }

    // A wrapped bullet continues the previous item.
    if (current && current.items.length > 0) {
      current.items[current.items.length - 1] += ` ${line}`;
    }
  }

  return sections.filter((section) => section.items.length > 0);
}

export function plainChangelog(body: string | null | undefined): string[] {
  if (!body) return [];
  return body
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));
}
