<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { TableActionState } from "$lib/states/table-actions.svelte";

  import { columns, type ServiceTableMeta } from "./columns";
  import { createDeleteServiceMutation, createServicesQuery } from "./queries";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");

  const queryClient = useQueryClient();

  const deleteServiceMutation = createDeleteServiceMutation(queryClient);

  const servicesQuery = createServicesQuery();

  const tableActions = new TableActionState<number>();

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
    get meta() {
      return {
        onEdit: tableActions.openEdit,
        onDelete: tableActions.openDelete,
      } satisfies ServiceTableMeta;
    },
    initialSorting: [{ id: "name", desc: true }],
  });

  function confirmDelete() {
    if (tableActions.deleteId !== null) {
      deleteServiceMutation.mutate(tableActions.deleteId, {
        onSuccess: tableActions.resetDelete,
      });
    }
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full flex items-center gap-2">
      <Input
        placeholder="Пошук за кодом, назвою, нормо-годинами або ціною..."
        disabled={!servicesQuery.isSuccess}
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        variant="outline"
        disabled={!servicesQuery.isSuccess}
        onclick={tableActions.openCreate}
      >
        Додати <PlusIcon />
      </Button>
    </div>
  {/snippet}
  {#if servicesQuery.isError}
    <FetchErrorDisplay
      message={servicesQuery.error?.message}
      onRetry={servicesQuery.refetch}
      class="mt-10"
    />
  {:else if servicesQuery.isLoading}
    <Skeleton class="mt-4 h-50 w-full rounded-lg" />
  {:else}
    <DataTable
      {table}
      emptyMessage="Сервісів не знайдено..."
      headerClasses={{ name: "w-full" }}
      cellClasses={{ actions: "p-0" }}
      class="mt-4 rounded-sm shadow-xs bg-card"
    />
  {/if}
</PageLayout>

<ServiceMutationSheet
  bind:open={tableActions.isMutationOpen}
  serviceId={tableActions.editId}
/>
<DeleteDialog
  bind:open={tableActions.isDeleteOpen}
  onDelete={confirmDelete}
  description="Цю дію неможливо відмінити."
/>
