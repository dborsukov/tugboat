<script lang="ts">
  import { prettifyIsoDateString } from "$lib/utils";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import { goto } from "$app/navigation";

  import { type Client } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import Button from "$lib/components/ui/button/button.svelte";

  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { client }: { client: Client } = $props();
  let cars = $derived(appStore.cars.filter((c) => c.client_id === client.id));
</script>

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
        <h1 class="text-2xl font-semibold">{client?.name}</h1>
        <p class="text-muted-foreground">ID: {client?.id}</p>
      </div>
    </div>
    <div class="border p-4 mt-4 rounded grid gap-3">
      <div class="flex items-center justify-between border-b border-muted pb-3">
        <span class="flex items-center gap-2 text-muted-foreground">
          <PhoneIcon class="size-4 text-muted-foreground/70" /> Номер
        </span>
        <span
          class="tracking-wide max-w-45 truncate select-all"
          title={client.phone}
        >
          {client.phone}
        </span>
      </div>
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-2 text-muted-foreground">
          <CalendarPlusIcon class="size-4 text-muted-foreground/70" /> Створено
        </span>
        <span class="tracking-wide max-w-45 truncate">
          {prettifyIsoDateString(client.created_at, "long")}
        </span>
      </div>
    </div>
    {#if cars.length != 0}
      <p class="mt-4 font-semibold text-2xl text-center">Автомобілі</p>
      <div class="flex flex-wrap gap-4 mt-4">
        {#each cars as car}
          <a href="/cars/{car.id}" class="flex-1 min-w-1/3">
            <CarCard {car} short />
          </a>
        {/each}
      </div>
    {/if}
  </div>
</PageLayout>
