<script lang="ts">
  import { prettifyIsoDateString } from "$lib/utils";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import HashIcon from "@lucide/svelte/icons/hash";
  import SquarePenIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
  import UserIcon from "@lucide/svelte/icons/user";

  import { goto } from "$app/navigation";

  import { commands, type Car } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Badge } from "$lib/components/ui/badge";
  import ButtonGroup from "$lib/components/ui/button-group/button-group.svelte";
  import Button from "$lib/components/ui/button/button.svelte";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import CarMutationSheet from "../car-mutation-sheet.svelte";
  import LogoLoader from "../logo-loader.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { car }: { car: Car } = $props();
  let owner = $derived(appStore.clients.find((c) => c.id === car.client_id));
  let brand = $derived(appStore.car_brands.find((b) => b.id === car.brand_id));
  let slug = $derived(brand != undefined ? brand.icon_slug : "");

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  async function handleDelete() {
    try {
      let res = await commands.deleteCar(car.id);
      if (res.status === "ok") {
        appStore.removeCar(car.id);
        await goto("/cars");
      } else {
        // TODO: show error to user
        alert(res.error);
      }
    } catch (err) {
      // TODO: show error to user
      alert(`Critical Tauri panic or IPC error: ${err}`);
    }
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center justify-between">
      <Button variant="secondary" onclick={() => goto("/cars")}>
        <ChevronLeftIcon /> Список
      </Button>
      <DropdownMenu {car} />
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
            {car.model}
          </div>
        </div>
        <div class="flex flex-col gap-1 justify-between items-end">
          <Badge variant="outline" class="text-lg px-3 py-3.5 bg-background">
            {car.year}
          </Badge>
          <p class="text-muted-foreground">ID: {car.id}</p>
        </div>
      </div>
    </div>
    <div class="border p-4 mt-4 rounded grid gap-3">
      <div class="flex items-center justify-between border-b border-muted pb-3">
        <span class="text-muted-foreground flex items-center gap-2">
          <UserIcon class="size-4 text-muted-foreground/70" /> Власник
        </span>
        {#if owner != undefined}
          <span class="max-w-45 truncate" title={owner.name}>
            {owner.name}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between border-b border-muted pb-3">
        <span class="text-muted-foreground flex items-center gap-2">
          <HashIcon class="size-4 text-muted-foreground/70" /> Реєстрація
        </span>
        {#if car.registration}
          <span
            class="font-mono font-bold uppercase tracking-wider bg-muted px-2 py-0.5 rounded text-sm border"
          >
            {car.registration}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between border-b border-muted pb-3">
        <span class="text-muted-foreground flex items-center gap-2">
          <FingerprintIcon class="size-4 text-muted-foreground/70" /> VIN
        </span>
        {#if car.vin}
          <span
            class="font-mono font-bold uppercase tracking-wider bg-muted px-2 py-0.5 rounded text-sm border"
          >
            {car.vin}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between">
        <span class="flex items-center gap-2 text-muted-foreground">
          <CalendarPlusIcon class="size-4 text-muted-foreground/70" /> Створено
        </span>
        <span class="max-w-45 truncate select-all">
          {prettifyIsoDateString(car.created_at, "long")}
        </span>
      </div>
    </div>
    <div class="border rounded p-4 mt-4">Invoices here...</div>
  </div>
</PageLayout>
