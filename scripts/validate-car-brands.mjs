#!/usr/bin/env node
/**
 * Validates the bundled car catalog (`src-tauri/data/car_brands.json`).
 *
 * The catalog is the source of truth for the brand picker: `slug` keys the
 * database rows and names the logo file, `name` is what the shop sees, `aliases`
 * are the alternative spellings search matches, and `models` carry their own
 * aliases. A typo here reaches the running app, so every commit checks it.
 *
 * `iconSlug` is optional and only written when a brand's logo lives under
 * another name (or is shared with a sibling brand) — writing the slug into it
 * is rejected, so the two can never drift apart.
 *
 * Every brand must own a `static/car_brand_logos/<slug>.png`, and every file in
 * that directory must belong to a brand: a logo no brand references is either a
 * leftover from a removed brand or a typo in the file name, and both mean the
 * picker shows the `?` placeholder for a brand that looks like it has a logo.
 *
 * Ids are not part of the catalog: the database assigns them.
 *
 * Runtime is lenient (serde ignores unknown fields, the brand sync logs
 * and skips a bad row instead of failing), so the strict checks live here.
 */
import { readdirSync, readFileSync } from "node:fs";

const DEFAULT_FILES = ["src-tauri/data/car_brands.json"];
const SCHEMA_FILE = "src-tauri/data/car_brands.schema.json";
const UNKNOWN_BRAND_SLUG = "unknown-brand";

const LOGO_DIR = "static/car_brand_logos";
const LOGO_EXTENSION = ".png";

/**
 * The sentinel is what the picker falls back to for an unrecognised make, so it
 * deliberately renders the "?" placeholder instead of an image. Drop it from
 * this set to require a logo for it too.
 */
const LOGO_EXEMPT_SLUGS = new Set([UNKNOWN_BRAND_SLUG]);

/** Slug and icon slug end up in URLs and file names. */
const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const ICON_SLUG_PATTERN = /^[a-z0-9-]+$/;

/** Mirrors MAX_NAME_LENGTH in brands.rs. */
const MAX_NAME_LENGTH = 60;

const BRAND_FIELDS = new Set(["slug", "name", "iconSlug", "aliases", "models"]);
const MODEL_FIELDS = new Set(["name", "aliases"]);

const args = process.argv.slice(2).filter((arg) => !arg.startsWith("-"));

/** The schema is validated on its own, it is not a catalog. */
const isSchemaFile = (file) =>
  file.replace(/\\/g, "/").endsWith("car_brands.schema.json");

// The hook passes every file it matched, which includes logos. Only JSON files
// are read as catalogs, and when none was named — a logo-only or schema-only
// change — the bundled catalog is still cross-checked against the logo folder.
const namedCatalogs = args.filter(
  (file) => !isSchemaFile(file) && file.endsWith(".json"),
);
const targets = namedCatalogs.length > 0 ? namedCatalogs : DEFAULT_FILES;

const problems = [];

function fail(where, message) {
  problems.push(`${where}: ${message}`);
}

/** Shapes the value for an error message without dumping the whole object. */
function describe(value) {
  const json = JSON.stringify(value);
  return json === undefined ? String(value) : json;
}

const nameKey = (value) => value.trim().toLowerCase();

/**
 * Aliases and model aliases obey the same rules: a non-empty list of distinct,
 * trimmed spellings that add something beyond the name they belong to.
 */
function checkAliases(aliases, where, label, ownerNames) {
  if (aliases === undefined) return;

  if (!Array.isArray(aliases) || aliases.length === 0) {
    fail(where, `${label} must be a non-empty array when present`);
    return;
  }

  const seen = new Set();

  for (const alias of aliases) {
    if (typeof alias !== "string" || alias.trim() === "") {
      fail(
        where,
        `${label} must be a non-empty string, got ${describe(alias)}`,
      );
      continue;
    }

    if (alias !== alias.trim()) {
      fail(where, `${label} ${describe(alias)} has surrounding whitespace`);
    }

    const key = nameKey(alias);
    if (ownerNames.has(key)) {
      fail(where, `${label} ${describe(alias)} repeats the name it belongs to`);
    }

    if (seen.has(key)) {
      fail(where, `${label} ${describe(alias)} is listed twice`);
    }
    seen.add(key);
  }
}

function checkModels(models, where) {
  if (models === undefined) return;

  if (!Array.isArray(models) || models.length === 0) {
    fail(where, "models must be a non-empty array when present");
    return;
  }

  const names = new Map();

  models.forEach((model, index) => {
    const modelWhere = `${where} → models[${index}]`;

    if (typeof model !== "object" || model === null || Array.isArray(model)) {
      fail(
        modelWhere,
        `must be an object like { "name": "Camry" }, got ${describe(model)}`,
      );
      return;
    }

    for (const field of Object.keys(model)) {
      if (!MODEL_FIELDS.has(field)) {
        fail(
          modelWhere,
          `unknown field ${describe(field)} (allowed: name, aliases)`,
        );
      }
    }

    if (typeof model.name !== "string" || model.name.trim() === "") {
      fail(
        modelWhere,
        `name must be a non-empty string, got ${describe(model.name)}`,
      );
      return;
    }

    if (model.name !== model.name.trim()) {
      fail(
        modelWhere,
        `name ${describe(model.name)} has surrounding whitespace`,
      );
    }

    const key = nameKey(model.name);
    if (names.has(key)) {
      fail(modelWhere, `duplicates model ${describe(names.get(key))}`);
    } else {
      names.set(key, model.name);
    }

    checkAliases(model.aliases, modelWhere, "alias", new Set([key]));
  });
}

function checkCatalog(catalog, file) {
  if (!Array.isArray(catalog) || catalog.length === 0) {
    fail(file, "must be a non-empty array of brands");
    return;
  }

  const slugs = new Map();
  const names = new Map();

  catalog.forEach((brand, index) => {
    const where = `${file} → [${index}]`;

    if (typeof brand !== "object" || brand === null || Array.isArray(brand)) {
      fail(where, `must be an object, got ${describe(brand)}`);
      return;
    }

    const label =
      typeof brand.slug === "string" ? `${where} (${brand.slug})` : where;

    for (const field of Object.keys(brand)) {
      if (!BRAND_FIELDS.has(field)) {
        const hint =
          field === "id"
            ? " — ids are assigned by the database, the catalog is keyed by slug"
            : field === "modelAliases"
              ? " — put aliases inside the model entry instead"
              : " (allowed: slug, name, iconSlug, aliases, models)";
        fail(label, `unknown field ${describe(field)}${hint}`);
      }
    }

    if (typeof brand.slug !== "string" || !SLUG_PATTERN.test(brand.slug)) {
      fail(
        label,
        `slug must be lowercase kebab-case, got ${describe(brand.slug)}`,
      );
    } else if (slugs.has(brand.slug)) {
      fail(label, `slug duplicates ${slugs.get(brand.slug)}`);
    } else {
      slugs.set(brand.slug, label);
    }

    if (typeof brand.name !== "string" || brand.name.trim() === "") {
      fail(
        label,
        `name must be a non-empty string, got ${describe(brand.name)}`,
      );
    } else {
      if (brand.name !== brand.name.trim()) {
        fail(label, `name ${describe(brand.name)} has surrounding whitespace`);
      }
      if (brand.name.length > MAX_NAME_LENGTH) {
        fail(label, `name is longer than ${MAX_NAME_LENGTH} characters`);
      }
      const key = nameKey(brand.name);
      if (names.has(key)) {
        fail(label, `name duplicates ${names.get(key)}`);
      } else {
        names.set(key, label);
      }
    }

    // Optional: the slug already names the logo, so this is only written when
    // a brand points at a differently named (or shared) file.
    if (brand.iconSlug !== undefined) {
      if (
        typeof brand.iconSlug !== "string" ||
        !ICON_SLUG_PATTERN.test(brand.iconSlug)
      ) {
        fail(
          label,
          `iconSlug must be a file-name-safe slug, got ${describe(brand.iconSlug)}`,
        );
      } else if (brand.iconSlug === brand.slug) {
        fail(
          label,
          `iconSlug ${describe(brand.iconSlug)} repeats the slug — omit it, the slug already names the logo`,
        );
      }
    }

    checkAliases(
      brand.aliases,
      label,
      "alias",
      // Only the name counts: the slug is a valid Latin spelling of a Cyrillic
      // name, so "Izh" for «Іж» is exactly what an alias is for.
      typeof brand.name === "string"
        ? new Set([nameKey(brand.name)])
        : new Set(),
    );
    checkModels(brand.models, label);
  });

  if (!slugs.has(UNKNOWN_BRAND_SLUG)) {
    fail(
      file,
      `the ${describe(UNKNOWN_BRAND_SLUG)} sentinel brand is required`,
    );
  }
}

/**
 * Cross-checks the catalog against the logo directory in both directions.
 *
 * `iconSlug` wins over `slug` when present, mirroring `icon_slug` in brands.rs,
 * and several brands may share one file, so the expectation is built the other
 * way round: logo stem → the brands that ask for it.
 *
 * @param {Map<string, string | undefined>} brands slug → iconSlug
 */
function checkLogos(brands) {
  const wanted = new Map();

  for (const [slug, iconSlug] of brands) {
    if (LOGO_EXEMPT_SLUGS.has(slug)) continue;

    const stem = iconSlug ?? slug;
    const owners = wanted.get(stem);

    if (owners === undefined) {
      wanted.set(stem, [slug]);
    } else {
      owners.push(slug);
    }
  }

  let entries;
  try {
    entries = readdirSync(LOGO_DIR);
  } catch (error) {
    fail(LOGO_DIR, `cannot be read: ${error.message}`);
    return 0;
  }

  const present = new Set();

  for (const entry of entries) {
    // Editor and OS droppings are not logos.
    if (entry.startsWith(".")) continue;

    if (!entry.endsWith(LOGO_EXTENSION)) {
      fail(
        `${LOGO_DIR}/${entry}`,
        `is not a ${LOGO_EXTENSION} file — the loader only requests <slug>${LOGO_EXTENSION}`,
      );
      continue;
    }

    present.add(entry.slice(0, -LOGO_EXTENSION.length));
  }

  for (const [stem, owners] of wanted) {
    if (present.has(stem)) continue;

    const needs =
      owners.length === 1
        ? `brand ${describe(owners[0])} needs it`
        : `brands ${owners.map(describe).join(", ")} need it`;

    fail(`${LOGO_DIR}/${stem}${LOGO_EXTENSION}`, `is missing, but ${needs}`);
  }

  for (const stem of present) {
    if (wanted.has(stem)) continue;

    fail(
      `${LOGO_DIR}/${stem}${LOGO_EXTENSION}`,
      "is not used by any brand in the catalog",
    );
  }

  return present.size;
}

/**
 * The schema only feeds editor hints, but a stale one would underline valid
 * data, so it has to agree with the rules enforced here. It stays optional:
 * delete the file and nothing but the hints goes away.
 */
function checkSchema() {
  let schema;
  try {
    schema = JSON.parse(readFileSync(SCHEMA_FILE, "utf8"));
  } catch (error) {
    if (error.code !== "ENOENT") {
      fail(SCHEMA_FILE, `cannot be read: ${error.message}`);
    }
    return;
  }

  const shapes = [
    { name: "brand", fields: BRAND_FIELDS, required: ["slug", "name"] },
    { name: "model", fields: MODEL_FIELDS, required: ["name"] },
  ];

  for (const { name, fields, required } of shapes) {
    const shape = schema.definitions?.[name];

    if (shape === undefined) {
      fail(SCHEMA_FILE, `definitions.${name} is missing`);
      continue;
    }

    if (shape.additionalProperties !== false) {
      fail(
        SCHEMA_FILE,
        `definitions.${name} must set additionalProperties to false`,
      );
    }

    const declared = new Set(Object.keys(shape.properties ?? {}));
    const missing = [...fields].filter((field) => !declared.has(field));
    const unknown = [...declared].filter((field) => !fields.has(field));

    if (missing.length > 0) {
      fail(
        SCHEMA_FILE,
        `definitions.${name} does not declare ${missing.join(", ")}`,
      );
    }
    if (unknown.length > 0) {
      fail(
        SCHEMA_FILE,
        `definitions.${name} declares ${unknown.join(", ")}, which the rules here reject`,
      );
    }

    for (const field of required) {
      if (!(shape.required ?? []).includes(field)) {
        fail(
          SCHEMA_FILE,
          `definitions.${name} must require ${describe(field)}`,
        );
      }
    }
  }

  if (schema.contains?.properties?.slug?.const !== UNKNOWN_BRAND_SLUG) {
    fail(
      SCHEMA_FILE,
      `must require the ${describe(UNKNOWN_BRAND_SLUG)} sentinel`,
    );
  }
}

checkSchema();

let checked = 0;
let brands = 0;
let models = 0;

/** slug → iconSlug from every catalog that parsed, for the logo cross-check. */
const catalogBrands = new Map();

for (const file of targets) {
  let raw;
  try {
    raw = readFileSync(file, "utf8");
  } catch (error) {
    fail(file, `cannot be read: ${error.message}`);
    continue;
  }

  if (raw.charCodeAt(0) === 0xfeff) {
    fail(file, "must not start with a byte order mark");
  }

  let catalog;
  try {
    catalog = JSON.parse(raw);
  } catch (error) {
    fail(file, `is not valid JSON: ${error.message}`);
    continue;
  }

  checkCatalog(catalog, file);

  checked += 1;
  brands += Array.isArray(catalog) ? catalog.length : 0;
  models += Array.isArray(catalog)
    ? catalog.reduce((total, brand) => total + (brand?.models?.length ?? 0), 0)
    : 0;

  // checkCatalog already reported any malformed entry, so this only collects
  // what it could read; the logo check would otherwise be noise on top of it.
  if (Array.isArray(catalog)) {
    for (const brand of catalog) {
      if (typeof brand?.slug !== "string" || catalogBrands.has(brand.slug)) {
        continue;
      }
      catalogBrands.set(
        brand.slug,
        typeof brand.iconSlug === "string" ? brand.iconSlug : undefined,
      );
    }
  }
}

// Skipped when nothing parsed: an empty expectation would report every logo on
// disk as unused and bury the real error.
let logos = 0;
if (catalogBrands.size > 0) {
  logos = checkLogos(catalogBrands);
}

if (problems.length > 0) {
  console.error(`✖ car catalog: ${problems.length} problem(s)\n`);
  for (const problem of problems) console.error(`  ${problem}`);
  console.error("");
  process.exit(1);
}

console.log(
  `✔ car catalog: ${checked} file(s), ${brands} brands, ${models} models, ` +
    `${logos} logos — valid`,
);
