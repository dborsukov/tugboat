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
 * scroll to and highlight it. Most entries live on the main settings pages;
 * the tax fields are also indexed on the business-data sub-page.
 */
export const settingsSearchIndex: SettingsSearchItem[] = [
  {
    id: "settings-timezone",
    route: "/settings/general",
    label: "Часовий пояс",
    keywords:
      "час часовий пояс загальні timezone time zone годинник gmt utc зміна часу",
    hint: "Загальні",
  },
  {
    id: "settings-default-phone-country",
    route: "/settings/general",
    label: "Країна за замовчуванням для телефонів",
    keywords:
      "телефон номер телефону код країни країна загальні phone country code dial",
    hint: "Загальні",
  },
  {
    id: "settings-documents-dir",
    route: "/settings/general",
    label: "Тека документів",
    keywords:
      "тека документи збереження документів шлях папка наряд замовлення акт виконаних робіт pdf xlsx загальні documents folder",
    hint: "Загальні",
  },
  {
    id: "settings-company-info-entry",
    route: "/settings/general",
    label: "Дані бізнесу",
    keywords:
      "дані бізнесу реквізити організація компанія фоп єдрпоу рнокпп ібан банк адреса керівник пдв документи рахунок акт дані загальні company details",
    hint: "Загальні",
  },
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
    id: "settings-vat-payer",
    route: "/settings/general/company-info",
    label: "Платник ПДВ",
    keywords:
      "пдв податок платник податкова vat tax payer оподаткування дані бізнесу",
    hint: "Дані бізнесу",
  },
  {
    id: "settings-vat-rate",
    route: "/settings/general/company-info",
    label: "Ставка ПДВ",
    keywords:
      "пдв податок ставка податкова vat tax відсоток ціни з пдв без пдв замовлення оподаткування",
    hint: "Дані бізнесу",
  },
  {
    id: "settings-vat-inclusive",
    route: "/settings/general/company-info",
    label: "Ціни включають ПДВ",
    keywords:
      "пдв податок ціни з пдв включено vat tax inclusive замовлення оподаткування дані бізнесу",
    hint: "Дані бізнесу",
  },
  {
    id: "settings-rate-variants-entry",
    route: "/settings/workorders",
    label: "Тарифи на замовлення",
    keywords: "тариф тарифи ставка розцінки rate замовлення",
    hint: "Замовлення",
  },
  {
    id: "settings-car-brands-entry",
    route: "/settings/cars",
    label: "Марки автомобілів",
    keywords:
      "марка марки автомобіль авто бренд brand виробник власна марка довідник",
    hint: "Автомобілі",
  },
  {
    id: "settings-vin-templates-entry",
    route: "/settings/cars",
    label: "Шаблони пошуку VIN-кодів",
    keywords: "vin код автомобіль авто пошук перевірка шаблон url template",
    hint: "Автомобілі",
  },
  {
    id: "settings-plate-templates-entry",
    route: "/settings/cars",
    label: "Шаблони пошуку номерних знаків",
    keywords:
      "номерний знак номер авто автомобіль пошук перевірка шаблон plate license url template",
    hint: "Автомобілі",
  },
  {
    id: "settings-expense-categories-entry",
    route: "/settings/finance",
    label: "Категорії витрат",
    keywords:
      "витрати категорія категорії витрат фінанси колір кольори паливо оренда зарплата expense category finance",
    hint: "Фінанси",
  },
];
