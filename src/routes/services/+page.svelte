<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import DataTableBulkActions from "$lib/components/data-table/data-table-bulk-actions.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { TableActionState } from "$lib/states/table-actions.svelte";

  import { columns, type ServiceTableMeta } from "./columns";
  import {
    createDeleteServiceMutation,
    createDeleteServicesMutation,
    createServicesQuery,
  } from "./queries";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");

  const queryClient = useQueryClient();

  const deleteServiceMutation = createDeleteServiceMutation(queryClient);
  const deleteServicesMutation = createDeleteServicesMutation(queryClient);

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
    getRowId: (row) => String(row.id),
    initialSorting: [{ id: "name", desc: true }],
  });

  const selectedServiceIds = $derived(
    table.getFilteredSelectedRowModel().rows.map((row) => row.original.id),
  );

  function pluralizeServices(count: number) {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return "сервіс";
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
      return "сервіси";
    }
    return "сервісів";
  }

  function confirmDelete() {
    if (tableActions.deleteId !== null) {
      const id = tableActions.deleteId;
      deleteServiceMutation.mutate(id, {
        onSuccess: () => {
          tableActions.resetDelete();
          table.setRowSelection((prev) => {
            const next = { ...prev };
            delete next[String(id)];
            return next;
          });
        },
      });
    }
  }

  function confirmBulkDelete() {
    if (tableActions.bulkDeleteIds.length > 0) {
      deleteServicesMutation.mutate([...tableActions.bulkDeleteIds], {
        onSuccess: () => {
          tableActions.resetBulkDelete();
          table.resetRowSelection();
        },
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
      headerClasses={{ name: "w-full", select: "w-10 p-0" }}
      cellClasses={{ actions: "p-0", select: "p-0" }}
      autoPageSize
      class="mt-4 rounded shadow-xs bg-card"
    >
      {#snippet selectionActions()}
        <DataTableBulkActions
          onDelete={() => tableActions.openBulkDelete(selectedServiceIds)}
        />
      {/snippet}
    </DataTable>
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
<DeleteDialog
  bind:open={tableActions.isBulkDeleteOpen}
  onDelete={confirmBulkDelete}
  description={`Видалити ${tableActions.bulkDeleteIds.length} ${pluralizeServices(tableActions.bulkDeleteIds.length)}? Цю дію неможливо відмінити.`}
/>
