export type SettingsSearchItem = {
  /** Item id (anchor) */
  id: string;
  /** Route the item lives on. */
  route: string;
  /** Primary label shown in the results. */
  label: string;
  /** Extra searchable text (lowercase words). */
  keywords?: string;
  /** Short context line shown under the label. */
  hint?: string;
};

/**
 * Registry of every searchable settings section. The `id` of each entry must
 * match an element id rendered on the corresponding page so the search can
 * scroll to and highlight it.
 */
export const settingsSearchIndex: SettingsSearchItem[] = [
  {
    id: "settings-theme",
    route: "/settings/lookandfeel",
    label: "Тема",
    keywords:
      "зовнішній вигляд оформлення theme світла темна системна light dark",
    hint: "Зовнішній вигляд",
  },
  {
    id: "settings-ui-scale",
    route: "/settings/lookandfeel",
    label: "Масштаб інтерфейсу",
    keywords: "розмір шрифт zoom зум інтерфейс ui scale зовнішній вигляд",
    hint: "Зовнішній вигляд",
  },
  {
    id: "settings-markup",
    route: "/settings/workorders",
    label: "Типова націнка",
    keywords: "націнка запчастини markup відсоток замовлення",
    hint: "Замовлення",
  },
  {
    id: "settings-rate-variants-entry",
    route: "/settings/workorders",
    label: "Тарифи на замовлення",
    keywords: "тариф тарифи ставка розцінки rate замовлення",
    hint: "Замовлення",
  },
  {
    id: "settings-rate-variants",
    route: "/settings/workorders/rate-variants",
    label: "Список тарифів",
    keywords: "тариф тарифи ставка розцінки rate список",
    hint: "Замовлення",
  },
];
