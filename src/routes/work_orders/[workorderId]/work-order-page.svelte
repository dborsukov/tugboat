<script lang="ts">
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import { goto } from "$app/navigation";

  import { type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/utils";

  import Button from "$lib/components/ui/button/button.svelte";

  import PageLayout from "$lib/components/page-layout.svelte";

  import DropdownMenu from "./dropdown-menu.svelte";

  let { work_order }: { work_order: WorkOrder } = $props();
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center justify-between">
      <Button variant="secondary" onclick={() => goto("/work_orders")}>
        <ChevronLeftIcon /> Список
      </Button>
      <DropdownMenu {work_order} />
    </div>
  {/snippet}
  <div class="h-full w-full p-4">
    <div class="border p-4 rounded">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-semibold">{work_order?.id}</h1>
      </div>
    </div>
  </div>
</PageLayout>
