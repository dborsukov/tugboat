/** @type {import("prettier").Config} */
export default {
  tabWidth: 2,

  plugins: [
    "prettier-plugin-svelte",
    "@ianvs/prettier-plugin-sort-imports",
    "prettier-plugin-sql",
  ],

  overrides: [
    { files: "*.svelte", options: { parser: "svelte" } },
    { files: "*.sql", options: { tabWidth: 4 } },
  ],

  importOrder: [
    "<BUILTIN_MODULES>",
    "",
    "<THIRD_PARTY_MODULES>",
    "",
    "^\\$app/(.*)$",
    "",
    "^\\@lucide/(.*)$",
    "",
    "^\\$lib/components/ui/(.*)$",
    "",
    "^\\$lib/components/(.*)$",
    "",
    "^\\$lib/(.*)$",
    "",
    "^\\.",
  ],
  importOrderCaseSensitive: false,
};
