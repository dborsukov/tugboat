import type { CompanyInfo, CompanyKind } from "$lib/bindings";
import { settingsState } from "$lib/states/settings.svelte";

/**
 * One incomplete required setting.
 *
 * Issues are derived, never stored: every consumer reads `getSetupIssues()`
 * and therefore updates the moment the settings query cache changes.
 */
export type SetupIssue = {
  /** Stable identifier for the rule. */
  key: "business_info";
  /** Human readable name, used by the settings row and the home card. */
  label: string;
  /** One line explaining why this matters, used by the home card. */
  description: string;
  /** Settings route that fixes the issue. */
  page: string;
  /** Element id of the section to open on that page. */
  section: string;
  /** The fields that are still missing, in display order. */
  fields: SetupIssueField[];
};

export type SetupIssueField = {
  /** Stable identifier inside the issue. */
  key: "identity" | "tax_code" | "address";
  /** Human readable field name. */
  label: string;
  /** Element id of the first input to focus for this field. */
  targetId: string;
};

const IDENTITY_LABEL = "Назва організації або ПІБ ФОП";
const TAX_CODE_LABEL = "Код ЄДРПОУ або РНОКПП";
const ADDRESS_LABEL = "Адреса";

function hasText(value: string | null | undefined): boolean {
  return (value ?? "").trim() !== "";
}

function identityField(
  info: CompanyInfo | null,
  kind: CompanyKind,
): SetupIssueField | null {
  if (kind === "company") {
    if (hasText(info?.name) || hasText(info?.legalName)) return null;

    return {
      key: "identity",
      label: IDENTITY_LABEL,
      targetId: "company-info-name",
    };
  }

  if (
    hasText(info?.surname) &&
    hasText(info?.firstName) &&
    hasText(info?.patronymic)
  ) {
    return null;
  }

  const targetId = !hasText(info?.surname)
    ? "company-info-surname"
    : !hasText(info?.firstName)
      ? "company-info-first-name"
      : "company-info-patronymic";

  return { key: "identity", label: IDENTITY_LABEL, targetId };
}

function taxCodeField(
  info: CompanyInfo | null,
  kind: CompanyKind,
): SetupIssueField | null {
  if (kind === "company") {
    return hasText(info?.edrpou)
      ? null
      : {
          key: "tax_code",
          label: TAX_CODE_LABEL,
          targetId: "company-info-edrpou",
        };
  }

  return hasText(info?.rnokpp)
    ? null
    : {
        key: "tax_code",
        label: TAX_CODE_LABEL,
        targetId: "company-info-rnokpp",
      };
}

function addressField(info: CompanyInfo | null): SetupIssueField | null {
  if (hasText(info?.physicalAddress) && hasText(info?.legalAddress))
    return null;

  return {
    key: "address",
    label: ADDRESS_LABEL,
    targetId: hasText(info?.physicalAddress)
      ? "company-info-legal-address"
      : "company-info-physical-address",
  };
}

function businessInfoFields(info: CompanyInfo | null): SetupIssueField[] {
  const kind = info?.kind ?? "fop";

  return [
    identityField(info, kind),
    taxCodeField(info, kind),
    addressField(info),
  ].filter((field): field is SetupIssueField => field !== null);
}

export function getSetupIssues(): SetupIssue[] {
  const settings = settingsState.settings;
  if (!settings) return [];

  const fields = businessInfoFields(settings.companyInfo);
  if (fields.length === 0) return [];

  return [
    {
      key: "business_info",
      label: "Дані бізнесу",
      description: "Потрібні для наряд-замовлення та акта",
      page: "/settings/general/company-info",
      section: "settings-company-info",
      fields,
    },
  ];
}
