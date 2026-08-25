<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import * as Item from "$lib/components/ui/item/index.js";
  import { Slider } from "$lib/components/ui/slider";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { createUpdateDefaultMarkupMutation } from "../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../settings-context";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateMarkupMutation = createUpdateDefaultMarkupMutation(queryClient);

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

  // ---------- Rate variants (managed on a separate page) ----------

  const rateVariantsCount = $derived(draft?.rateVariants.length ?? 0);

  function pluralTariffs(count: number): string {
    const n = Math.abs(count) % 100;
    const n1 = n % 10;
    if (n > 10 && n < 20) return `${count} тарифів`;
    if (n1 > 1 && n1 < 5) return `${count} тарифи`;
    if (n1 === 1) return `${count} тариф`;
    return `${count} тарифів`;
  }
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-12 border border-border rounded bg-card">
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
            {pluralTariffs(rateVariantsCount)}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
