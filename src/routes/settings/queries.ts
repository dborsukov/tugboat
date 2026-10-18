import { QueryClient } from "@tanstack/svelte-query";

import {
  commands,
  type CompanyInfo,
  type LicensePlateLookupTemplates,
  type RateVariants,
  type ThemeMode,
  type VinLookupTemplates,
} from "$lib/bindings";
import { createSettingsFieldMutation } from "$lib/states/settings.svelte";
import { unwrap } from "$lib/tools";

export { createSettingsQuery } from "$lib/states/settings.svelte";

export function createUpdateThemeMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(queryClient, "theme", (theme: ThemeMode) =>
    unwrap(commands.updateTheme(theme)),
  );
}

export function createUpdateUiScaleMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "uiScale",
    // FIXME: figure out why specta generates invalid bindings
    // specta generates "number | null" for some reason, even though
    // Rust always returns f64 - ignoring for now
    async (uiScale: number) =>
      (await unwrap(commands.updateUiScale(uiScale))) ?? uiScale,
  );
}

export function createUpdateDefaultMarkupMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "defaultMarkup",
    (defaultMarkup: string) =>
      unwrap(commands.updateDefaultMarkup(defaultMarkup)),
  );
}

export function createUpdateDefaultVatRateMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "defaultVatRate",
    (defaultVatRate: string) =>
      unwrap(commands.updateDefaultVatRate(defaultVatRate)),
  );
}

export function createUpdateDefaultPricesIncludeVatMutation(
  queryClient: QueryClient,
) {
  return createSettingsFieldMutation(
    queryClient,
    "defaultPricesIncludeVat",
    (defaultPricesIncludeVat: boolean) =>
      unwrap(commands.updateDefaultPricesIncludeVat(defaultPricesIncludeVat)),
  );
}

export function createUpdateRateVariantsMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "rateVariants",
    (rateVariants: RateVariants[]) =>
      unwrap(commands.updateRateVariants(rateVariants)),
  );
}

export function createUpdateVinLookupTemplatesMutation(
  queryClient: QueryClient,
) {
  return createSettingsFieldMutation(
    queryClient,
    "vinLookupTemplates",
    (vinLookupTemplates: VinLookupTemplates[]) =>
      unwrap(commands.updateVinLookupTemplates(vinLookupTemplates)),
  );
}

export function createUpdateLicensePlateLookupTemplatesMutation(
  queryClient: QueryClient,
) {
  return createSettingsFieldMutation(
    queryClient,
    "licensePlateLookupTemplates",
    (licensePlateLookupTemplates: LicensePlateLookupTemplates[]) =>
      unwrap(
        commands.updateLicensePlateLookupTemplates(licensePlateLookupTemplates),
      ),
  );
}

export function createUpdateTimezoneMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "timezone",
    (timezone: string) => unwrap(commands.updateTimezone(timezone)),
  );
}

export function createUpdateDefaultPhoneCountryCodeMutation(
  queryClient: QueryClient,
) {
  return createSettingsFieldMutation(
    queryClient,
    "defaultPhoneCountryCode",
    (defaultPhoneCountryCode: string) =>
      unwrap(commands.updateDefaultPhoneCountryCode(defaultPhoneCountryCode)),
  );
}

export function createUpdateCompanyInfoMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "companyInfo",
    (companyInfo: CompanyInfo) =>
      unwrap(commands.updateCompanyInfo(companyInfo)),
  );
}
