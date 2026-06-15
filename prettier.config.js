/** @type {import("prettier").Config} */
export default {
  tabWidth: 2,

  plugins: ["prettier-plugin-svelte", "@ianvs/prettier-plugin-sort-imports"],

  overrides: [{ files: "*.svelte", options: { parser: "svelte" } }],

  importOrder: [
    "<BUILTIN_MODULES>",
    "",
    "<THIRD_PARTY_MODULES>",
    "",
    "^\\@lucide/(.*)$",
    "",
    "^\\$app/(.*)$",
    "",
    "^\\$lib/bindings(.*)$",
    "",
    "^\\$lib/stores/(.*)$",
    "",
    "^\\$lib/components/ui/(.*)$",
    "",
    "^\\$lib/components/(.*)$",
    "",
    "^[./]",
  ],
  importOrderCaseSensitive: false,
};
