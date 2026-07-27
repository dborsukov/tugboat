<script lang="ts">
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");
  let isSheetOpen: boolean = $state(false);

  let table = buildTable({
    get data() {
      return appStore.services;
    },
    get columns() {
      return columns;
    },
    get searchTerm() {
      return inputValue;
    },
    initialSorting: [{ id: "name", desc: true }],
  });
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full pl-1 flex items-center gap-2">
      <Input
        placeholder="Пошук за кодом, назвою або нормо-годинами..."
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        variant="outline"
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        Додати <PlusIcon />
      </Button>
      <ServiceMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  <div class="p-4">
    <DataTable
      {table}
      emptyMessage="Сервісів не знайдено..."
      headerClasses={{ name: "w-full" }}
      class="rounded-sm shadow-xs"
    />
  </div>
</PageLayout>
