<script lang="ts">
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";
  import { createServicesQuery } from "./queries";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");
  let isSheetOpen: boolean = $state(false);

  const servicesQuery = createServicesQuery();

  let table = buildTable({
    get data() {
      return servicesQuery.isSuccess ? servicesQuery.data : [];
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
  <ServiceMutationSheet bind:open={isSheetOpen} />
  {#snippet header()}
    <div class="h-full pl-1 flex items-center gap-2">
      <Input
        placeholder="Пошук за кодом, назвою або нормо-годинами..."
        disabled={!servicesQuery.isSuccess}
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        variant="outline"
        disabled={!servicesQuery.isSuccess}
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        Додати <PlusIcon />
      </Button>
    </div>
  {/snippet}
  <div class="p-4">
    {#if servicesQuery.isError}
      <FetchErrorDisplay
        message={servicesQuery.error?.message}
        onRetry={servicesQuery.refetch}
        class="mt-20"
      />
    {:else if servicesQuery.isLoading}
      <Skeleton class="h-50 w-full rounded-lg" />
    {:else}
      <DataTable
        {table}
        emptyMessage="Сервісів не знайдено..."
        headerClasses={{ name: "w-full" }}
        class="rounded-sm shadow-xs"
      />
    {/if}
  </div>
</PageLayout>
