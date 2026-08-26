<script lang="ts">
  import { goto } from "$app/navigation";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import * as Item from "$lib/components/ui/item/index.js";

  import PageLayout from "$lib/components/page-layout.svelte";

  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../settings-context";

  const { draft } = getSettingsContext();

  // ---------- VIN lookup templates (managed on a separate page) ----------

  const vinTemplatesCount = $derived(draft?.vinLookupTemplates.length ?? 0);

  // ---------- License plate lookup templates (managed on a separate page) ----------

  const plateTemplatesCount = $derived(
    draft?.licensePlateLookupTemplates.length ?? 0,
  );

  function pluralTemplates(count: number): string {
    const n = Math.abs(count) % 100;
    const n1 = n % 10;
    if (n > 10 && n < 20) return `${count} шаблонів`;
    if (n1 > 1 && n1 < 5) return `${count} шаблони`;
    if (n1 === 1) return `${count} шаблон`;
    return `${count} шаблонів`;
  }
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 border border-border rounded bg-card">
    <Item.Root
      id="settings-vin-templates-entry"
      role="button"
      tabindex={0}
      class="hover:cursor-pointer hover:bg-muted/40 focus-visible:bg-muted/40"
      onclick={() => goto("/settings/cars/vin-lookup-templates")}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          goto("/settings/cars/vin-lookup-templates");
        }
      }}
    >
      <Item.Content>
        <Item.Title>Шаблони пошуку VIN-кодів</Item.Title>
        <Item.Description
          >Сервіси для перевірки/пошуку автомобіля за VIN-кодом</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-3">
          <p class="text-sm text-muted-foreground whitespace-nowrap">
            {pluralTemplates(vinTemplatesCount)}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>

    <Item.Root
      id="settings-plate-templates-entry"
      role="button"
      tabindex={0}
      class="hover:cursor-pointer hover:bg-muted/40 focus-visible:bg-muted/40"
      onclick={() => goto("/settings/cars/license-plate-lookup-templates")}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          goto("/settings/cars/license-plate-lookup-templates");
        }
      }}
    >
      <Item.Content>
        <Item.Title>Шаблони пошуку номерних знаків</Item.Title>
        <Item.Description
          >Сервіси для перевірки/пошуку автомобіля за номерним знаком</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-3">
          <p class="text-sm text-muted-foreground whitespace-nowrap">
            {pluralTemplates(plateTemplatesCount)}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
