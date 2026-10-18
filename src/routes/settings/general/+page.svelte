<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { countries } from "svelte-tel-input";
  import type { Country } from "svelte-tel-input/types";

  import { goto } from "$app/navigation";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import { Badge } from "$lib/components/ui/badge";
  import * as Item from "$lib/components/ui/item/index.js";

  import ComboSelect from "$lib/components/combo-select.svelte";
  import CountryFlag from "$lib/components/country-flag.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { getCountryNameUk } from "$lib/tools";

  import {
    createUpdateDefaultPhoneCountryCodeMutation,
    createUpdateTimezoneMutation,
  } from "../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../settings-context";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateTimezoneMutation = createUpdateTimezoneMutation(queryClient);
  const updateDefaultPhoneCountryCodeMutation =
    createUpdateDefaultPhoneCountryCodeMutation(queryClient);

  // ---------- Timezone ----------

  const FALLBACK_TIMEZONES = [
    "UTC",
    "Europe/Kyiv",
    "Europe/Warsaw",
    "Europe/Berlin",
    "Europe/Paris",
    "Europe/London",
    "Europe/Madrid",
    "Europe/Rome",
    "Europe/Amsterdam",
    "Europe/Prague",
    "Europe/Bucharest",
    "Europe/Sofia",
    "Europe/Vienna",
    "Europe/Budapest",
    "Europe/Belgrade",
    "Europe/Stockholm",
    "Europe/Oslo",
    "Europe/Helsinki",
    "Europe/Riga",
    "Europe/Vilnius",
    "Europe/Tallinn",
    "Europe/Istanbul",
    "Asia/Almaty",
    "Asia/Tbilisi",
    "Asia/Yerevan",
    "America/New_York",
    "America/Chicago",
    "America/Denver",
    "America/Los_Angeles",
    "America/Toronto",
    "America/Sao_Paulo",
    "America/Mexico_City",
    "Africa/Cairo",
    "Africa/Johannesburg",
    "Asia/Dubai",
    "Asia/Tokyo",
    "Asia/Shanghai",
    "Asia/Seoul",
    "Asia/Singapore",
    "Asia/Bangkok",
    "Asia/Jakarta",
    "Asia/Kolkata",
    "Asia/Karachi",
    "Australia/Sydney",
    "Australia/Melbourne",
    "Australia/Perth",
    "Pacific/Auckland",
  ];

  function listTimeZones(): string[] {
    try {
      // Full IANA timezone list when the runtime supports it (Chromium-based
      // webviews do). Falls back to a curated list otherwise.
      return Intl.supportedValuesOf("timeZone");
    } catch {
      return FALLBACK_TIMEZONES;
    }
  }

  function timeZoneOffset(timeZone: string): string {
    try {
      const parts = new Intl.DateTimeFormat("uk-UA", {
        timeZone,
        timeZoneName: "longOffset",
      }).formatToParts(new Date());
      return parts.find((p) => p.type === "timeZoneName")?.value ?? "";
    } catch {
      return "";
    }
  }

  type TimeZoneItem = { id: string; offset: string };

  const timeZoneItems = $derived<TimeZoneItem[]>(
    listTimeZones()
      .map((id) => ({ id, offset: timeZoneOffset(id) }))
      .sort((a, b) => a.id.localeCompare(b.id)),
  );

  function onTimezoneChange(value: unknown) {
    if (!draft) return;
    const next = String(value);
    if (next === draft.timezone) return;
    draft.timezone = next;
    updateTimezoneMutation.mutate(next);
  }

  // ---------- Default phone country code ----------

  type CountryItem = Country & { ukName: string };

  const countryItems = $derived<CountryItem[]>(
    [...countries]
      .map((c) => ({ ...c, ukName: getCountryNameUk(c.iso2, c.name) }))
      .sort((a, b) => {
        if (a.iso2 === "UA") return -1;
        if (b.iso2 === "UA") return 1;
        return a.ukName.localeCompare(b.ukName, "uk");
      }),
  );

  function onCountryChange(value: unknown) {
    if (!draft) return;
    const next = String(value);
    if (next === draft.defaultPhoneCountryCode) return;
    draft.defaultPhoneCountryCode = next;
    updateDefaultPhoneCountryCodeMutation.mutate(next);
  }
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 border border-border rounded bg-card">
    <Item.Root id="settings-timezone">
      <Item.Content>
        <Item.Title>Часовий пояс</Item.Title>
        <Item.Description
          >Часовий пояс для форматування дат і часу</Item.Description
        >
        {#if updateTimezoneMutation.isError}
          <p class="text-xs text-destructive mt-1">
            Не вдалося зберегти часовий пояс:
            {updateTimezoneMutation.error?.message}
          </p>
        {/if}
      </Item.Content>
      <Item.Actions>
        <ComboSelect
          items={timeZoneItems}
          value={draft?.timezone ?? ""}
          getKey={(t) => t.id}
          getLabel={(t) => t.id}
          getSearchValue={(t) => `${t.id} ${t.offset}`}
          onValueChange={onTimezoneChange}
          triggerClass="w-64"
          contentClass="w-80 p-0"
        >
          {#snippet item(t)}
            <span class="flex-1 truncate">{t.id}</span>
            {#if t.offset}
              <span class="text-muted-foreground text-xs tabular-nums">
                {t.offset}
              </span>
            {/if}
          {/snippet}
          {#snippet trigger(selected)}
            <span class="flex min-w-0 items-center gap-2">
              <span class="flex-1 truncate">
                {selected ? selected.id : "Оберіть..."}
              </span>
              {#if selected?.offset}
                <span class="text-muted-foreground text-xs tabular-nums">
                  {selected.offset}
                </span>
              {/if}
            </span>
          {/snippet}
        </ComboSelect>
      </Item.Actions>
    </Item.Root>

    <Item.Root id="settings-default-phone-country">
      <Item.Content>
        <Item.Title>Країна за замовчуванням для телефонів</Item.Title>
        <Item.Description
          >Код країни який застосовується за замовчуванням для нових номерів</Item.Description
        >
        {#if updateDefaultPhoneCountryCodeMutation.isError}
          <p class="text-xs text-destructive mt-1">
            Не вдалося зберегти країну:
            {updateDefaultPhoneCountryCodeMutation.error?.message}
          </p>
        {/if}
      </Item.Content>
      <Item.Actions>
        <ComboSelect
          items={countryItems}
          value={draft?.defaultPhoneCountryCode ?? ""}
          getKey={(c) => c.iso2}
          getLabel={(c) => c.ukName}
          getSearchValue={(c) => `${c.ukName} ${c.name} +${c.dialCode}`}
          onValueChange={onCountryChange}
          triggerClass="w-64"
          contentClass="w-80 p-0"
        >
          {#snippet item(c)}
            <CountryFlag country={c.iso2} />
            <span class="flex-1 truncate">{c.ukName}</span>
            <span class="text-muted-foreground tabular-nums">
              +{c.dialCode}
            </span>
          {/snippet}
          {#snippet trigger(selected)}
            <span class="flex min-w-0 items-center gap-2">
              <CountryFlag country={selected?.iso2 ?? null} />
              <span class="flex-1 truncate">
                {selected ? selected.ukName : "Оберіть..."}
              </span>
              {#if selected}
                <span class="text-muted-foreground tabular-nums">
                  +{selected.dialCode}
                </span>
              {/if}
            </span>
          {/snippet}
        </ComboSelect>
      </Item.Actions>
    </Item.Root>

    <Item.Root
      id="settings-company-info-entry"
      role="button"
      tabindex={0}
      class="hover:cursor-pointer hover:bg-muted/40 focus-visible:bg-muted/40"
      onclick={() => goto("/settings/general/company-info")}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          goto("/settings/general/company-info");
        }
      }}
    >
      <Item.Content>
        <Item.Title>Дані бізнесу</Item.Title>
        <Item.Description
          >Дані вашого бізнесу для використання в документах</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <Badge variant={draft?.companyInfo?.isComplete ? "success" : "warning"}>
          {draft?.companyInfo?.isComplete ? "Заповнено" : "Не заповнено"}
        </Badge>
        <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
