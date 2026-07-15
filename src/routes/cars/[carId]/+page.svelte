<script lang="ts">
  import WorkOrderIcon from "@lucide/svelte/icons/briefcase-business";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import HashIcon from "@lucide/svelte/icons/hash";
  import UserIcon from "@lucide/svelte/icons/user";

  import { goto } from "$app/navigation";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/utils";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Details from "$lib/components/ui/details/index";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { data } = $props();
  let owner = $derived(
    appStore.clients.find((c) => c.id === data.car.client_id),
  );
  let brand = $derived(
    appStore.car_brands.find((b) => b.id === data.car.brand_id),
  );
  let slug = $derived(brand != undefined ? brand.icon_slug : "");

  let isSheetOpen: boolean = $state(false);
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center">
      <Button variant="secondary" onclick={() => goto("/cars")}>
        <ChevronLeftIcon /> Список
      </Button>
      <Button class="ml-auto mr-2" onclick={() => (isSheetOpen = !isSheetOpen)}>
        <WorkOrderIcon /> Нове замовлення
      </Button>
      <DropdownMenu car={data.car} />
      <WorkOrderMutationSheet bind:open={isSheetOpen} car_id={data.car.id} />
    </div>
  {/snippet}
  <div class="p-4">
    <div class="border p-4 rounded">
      <div class="flex flex-row justify-center items-center gap-4 space-y-0">
        <div class="h-15">
          <LogoLoader {slug} />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-xl font-semibold tracking-tight truncate">
            {#if brand != undefined}
              {brand.name}
            {:else}
              ?
            {/if}
          </div>
          <div class="text-lg tracking-tight mt-0.5">
            {data.car.model}
          </div>
        </div>
        <div class="flex flex-col gap-1 justify-between items-end">
          <Badge variant="outline" class="text-lg px-3 py-3.5 bg-background">
            {data.car.year}
          </Badge>
          <p class="text-muted-foreground">ID: {data.car.id}</p>
        </div>
      </div>
    </div>

    <Details.Card class="mt-4">
      <Details.Item>
        <Details.Title>
          <Details.Icon icon={UserIcon} /> Власник
        </Details.Title>
        <Details.Content>
          {#if owner != undefined}
            {owner.name}
          {:else}
            Не вказано
          {/if}
        </Details.Content>
      </Details.Item>

      <Details.Item>
        <Details.Title>
          <Details.Icon icon={HashIcon} /> Державний номер
        </Details.Title>
        <Details.Content>
          {#if owner != undefined}
            {data.car.registration}
          {:else}
            Не вказано
          {/if}
        </Details.Content>
      </Details.Item>

      <Details.Item>
        <Details.Title>
          <Details.Icon icon={FingerprintIcon} /> VIN-код
        </Details.Title>
        <Details.Content>
          {#if owner != undefined}
            {data.car.vin}
          {:else}
            Не вказано
          {/if}
        </Details.Content>
      </Details.Item>

      <Details.Item>
        <Details.Title>
          <Details.Icon icon={CalendarPlusIcon} /> Створено
        </Details.Title>
        <Details.Content>
          {prettifyIsoDateString(data.car.created_at, "long")}
        </Details.Content>
      </Details.Item>
    </Details.Card>

    <div class="border rounded p-4 mt-4">Invoices here...</div>
  </div>
</PageLayout>
