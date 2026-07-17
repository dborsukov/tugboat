<script lang="ts">
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import { goto } from "$app/navigation";

  import type { Client } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/utils";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Details from "$lib/components/ui/details/index";

  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import ClientBadge from "../client-badge.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { data } = $props();
  let client = $derived(
    appStore.clients.find((c) => c.id.toString() === data.id) as Client,
  );
  let cars = $derived(appStore.cars.filter((c) => c.client_id === client.id));
</script>

{#if client}
  <PageLayout>
    {#snippet header()}
      <div class="h-full px-4 flex items-center justify-between">
        <Button variant="secondary" onclick={() => goto("/clients")}>
          <ChevronLeftIcon /> Список
        </Button>
        <DropdownMenu {client} />
      </div>
    {/snippet}
    <div class="h-full w-full p-4">
      <div class="border p-4 rounded">
        <div class="flex items-center justify-between">
          <ClientBadge
            {client}
            class="text-xl font-semibold"
            iconSize="size-10"
          />
          <p class="text-muted-foreground">ID: {client.id}</p>
        </div>
      </div>
      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={PhoneIcon} /> Номер
          </Details.Title>
          <Details.Content>
            {client.phone}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(client.created_at, "long")}
          </Details.Content>
        </Details.Item>
      </Details.Card>

      {#if cars.length != 0}
        <p class="mt-4 font-semibold text-2xl text-center">Автомобілі</p>
        <div class="flex flex-wrap gap-4 mt-4">
          {#each cars as car (car.id)}
            <a href="/cars/{car.id}" class="flex-1 min-w-1/3">
              <CarCard {car} short />
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </PageLayout>
{/if}
