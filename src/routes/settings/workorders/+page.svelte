<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Slider } from "$lib/components/ui/slider";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { pluralizeUk } from "$lib/tools";

  import {
    createUpdateDefaultMarkupMutation,
    createUpdateDefaultPricesIncludeVatMutation,
    createUpdateDefaultVatRateMutation,
  } from "../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../settings-context";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateMarkupMutation = createUpdateDefaultMarkupMutation(queryClient);
  const updateVatRateMutation = createUpdateDefaultVatRateMutation(queryClient);
  const updatePricesIncludeVatMutation =
    createUpdateDefaultPricesIncludeVatMutation(queryClient);

  // ---------- Default markup ----------

  const markupPercent = $derived(draft ? Number(draft.defaultMarkup) : 0);

  function onMarkupChange(value: number) {
    if (!draft) return;
    draft.defaultMarkup = String(value);
  }

  function onMarkupCommit(value: number) {
    if (!draft) return;
    draft.defaultMarkup = String(value);
    // fires once per slider release, so it can persist immediately
    updateMarkupMutation.mutate(draft.defaultMarkup);
  }

  // ---------- VAT ----------

  // Free-text rate input: keep a local draft while typing, persist on blur.
  let vatRateInput = $state("20");
  let vatRateFocused = $state(false);

  $effect(() => {
    if (!vatRateFocused && draft) {
      vatRateInput = draft.defaultVatRate;
    }
  });

  function commitVatRate(raw: string) {
    if (!draft) return;
    const normalized = raw.trim().replace(",", ".");
    const value = Number(normalized);
    if (
      normalized === "" ||
      !Number.isFinite(value) ||
      value < 0 ||
      value > 100
    ) {
      vatRateInput = draft.defaultVatRate;
      return;
    }
    draft.defaultVatRate = normalized;
    vatRateInput = normalized;
    updateVatRateMutation.mutate(normalized);
  }

  function onVatRateKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      (e.currentTarget as HTMLInputElement).blur();
    }
  }

  // ---------- Rate variants (managed on a separate page) ----------

  const rateVariantsCount = $derived(draft?.rateVariants.length ?? 0);
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 border border-border rounded bg-card">
    <Item.Root
      id="settings-rate-variants-entry"
      role="button"
      tabindex={0}
      class="hover:cursor-pointer hover:bg-muted/40 focus-visible:bg-muted/40"
      onclick={() => goto("/settings/workorders/rate-variants")}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          goto("/settings/workorders/rate-variants");
        }
      }}
    >
      <Item.Content>
        <Item.Title>Тарифи на замовлення</Item.Title>
        <Item.Description
          >Список тарифів для розрахунку вартості замовлень</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-3">
          <p class="text-sm text-muted-foreground whitespace-nowrap">
            {rateVariantsCount}{" "}
            {pluralizeUk(rateVariantsCount, ["тариф", "тарифи", "тарифів"])}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>

    <Item.Root id="settings-markup">
      <Item.Content>
        <Item.Title>Типова націнка</Item.Title>
        <Item.Description
          >Націнка за замовчуванням для нових запчастин</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-4">
          <p>{markupPercent}%</p>
          <Slider
            type="single"
            value={markupPercent}
            onValueChange={onMarkupChange}
            onValueCommit={onMarkupCommit}
            min={0}
            max={300}
            step={1}
            class="w-40"
          />
        </div>
      </Item.Actions>
    </Item.Root>

    <Item.Root id="settings-vat-inclusive">
      <Item.Content>
        <Item.Title>Ціни включають ПДВ</Item.Title>
        <Item.Description
          >Вважати ціни в нових замовленнях такими, що вже включають ПДВ, за
          замовчуванням</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <Checkbox
          checked={draft?.defaultPricesIncludeVat ?? true}
          onCheckedChange={(checked) => {
            const next = checked === true;
            if (!draft) return;
            draft.defaultPricesIncludeVat = next;
            updatePricesIncludeVatMutation.mutate(next);
          }}
          aria-label="Ціни з ПДВ"
        />
      </Item.Actions>
    </Item.Root>

    <Item.Root id="settings-vat-rate">
      <Item.Content>
        <Item.Title>Ставка ПДВ</Item.Title>
        <Item.Description
          >Ставка за замовчуванням для нових замовлень</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-2">
          <Input
            bind:value={vatRateInput}
            type="text"
            inputmode="decimal"
            class="w-20 text-right"
            onfocus={() => (vatRateFocused = true)}
            onblur={() => {
              vatRateFocused = false;
              commitVatRate(vatRateInput);
            }}
            onkeydown={onVatRateKeydown}
          />
          <p>%</p>
        </div>
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
