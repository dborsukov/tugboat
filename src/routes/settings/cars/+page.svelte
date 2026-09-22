<script lang="ts">
  import { goto } from "$app/navigation";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import * as Item from "$lib/components/ui/item/index.js";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { pluralizeUk } from "$lib/tools";

  import { createCustomCarBrandsQuery } from "../../cars/queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../settings-context";

  const { draft } = getSettingsContext();

  // ---------- Custom car brands (managed on a separate page) ----------

  const customBrandsQuery = createCustomCarBrandsQuery();

  const customBrandsCount = $derived(customBrandsQuery.data?.length ?? 0);

  // ---------- VIN lookup templates (managed on a separate page) ----------

  const vinTemplatesCount = $derived(draft?.vinLookupTemplates.length ?? 0);

  // ---------- License plate lookup templates (managed on a separate page) ----------

  const plateTemplatesCount = $derived(
    draft?.licensePlateLookupTemplates.length ?? 0,
  );
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 border border-border rounded bg-card">
    <Item.Root
      id="settings-car-brands-entry"
      role="button"
      tabindex={0}
      class="hover:cursor-pointer hover:bg-muted/40 focus-visible:bg-muted/40"
      onclick={() => goto("/settings/cars/car-brands")}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          goto("/settings/cars/car-brands");
        }
      }}
    >
      <Item.Content>
        <Item.Title>Марки автомобілів</Item.Title>
        <Item.Description
          >Додайте власні марки яких бракує у вбудованому довіднику</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-3">
          <p class="text-sm text-muted-foreground whitespace-nowrap">
            {customBrandsCount}
            {pluralizeUk(customBrandsCount, ["марка", "марки", "марок"])}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>

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
            {vinTemplatesCount}{" "}
            {pluralizeUk(vinTemplatesCount, ["шаблон", "шаблони", "шаблонів"])}
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
            {plateTemplatesCount}{" "}
            {pluralizeUk(plateTemplatesCount, [
              "шаблон",
              "шаблони",
              "шаблонів",
            ])}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
