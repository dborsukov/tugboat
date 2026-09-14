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
  import { pluralizeUk } from "$lib/tools";

  import { columns, type WorkerTableMeta } from "./columns";
  import {
    createDeleteWorkerMutation,
    createDeleteWorkersMutation,
    createWorkersQuery,
  } from "./queries";
  import WorkerMutationSheet from "./worker-mutation-sheet.svelte";

  let inputValue: string = $state("");

  const queryClient = useQueryClient();

  const deleteWorkerMutation = createDeleteWorkerMutation(queryClient);
  const deleteWorkersMutation = createDeleteWorkersMutation(queryClient);

  const workersQuery = createWorkersQuery();

  const tableActions = new TableActionState<number>();

  const workers = $derived(workersQuery.isSuccess ? workersQuery.data : []);

  let table = buildTable({
    get data() {
      return workers;
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
      } satisfies WorkerTableMeta;
    },
    getRowId: (row) => String(row.id),
    initialSorting: [{ id: "name", desc: false }],
  });

  const selectedWorkerIds = $derived(
    table.getFilteredSelectedRowModel().rows.map((row) => row.original.id),
  );

  function confirmDelete() {
    if (tableActions.deleteId !== null) {
      const id = tableActions.deleteId;
      deleteWorkerMutation.mutate(id, {
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
      deleteWorkersMutation.mutate([...tableActions.bulkDeleteIds], {
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
        placeholder="Пошук за ім'ям або посадою..."
        disabled={!workersQuery.isSuccess}
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        variant="outline"
        disabled={!workersQuery.isSuccess}
        onclick={tableActions.openCreate}
      >
        Додати <PlusIcon />
      </Button>
    </div>
  {/snippet}
  {#if workersQuery.isError}
    <FetchErrorDisplay
      message={workersQuery.error?.message}
      onRetry={workersQuery.refetch}
      class="mt-10"
    />
  {:else if workersQuery.isLoading}
    <Skeleton class="mt-4 h-50 w-full rounded-lg" />
  {:else}
    <DataTable
      {table}
      emptyMessage="Працівників не знайдено..."
      headerClasses={{ name: "w-full", select: "w-10 p-0" }}
      cellClasses={{ actions: "p-0", select: "p-0", status: "p-0" }}
      autoPageSize
      class="mt-4 rounded shadow-xs bg-card"
    >
      {#snippet selectionActions()}
        <DataTableBulkActions
          onDelete={() => tableActions.openBulkDelete(selectedWorkerIds)}
        />
      {/snippet}
    </DataTable>
  {/if}
</PageLayout>

<WorkerMutationSheet
  bind:open={tableActions.isMutationOpen}
  workerId={tableActions.editId}
/>
<DeleteDialog
  bind:open={tableActions.isDeleteOpen}
  onDelete={confirmDelete}
  description="Працівника буде видалено безповоротно, а в замовленнях, де його вказано виконавцем, виконавця буде скинуто."
/>
<DeleteDialog
  bind:open={tableActions.isBulkDeleteOpen}
  onDelete={confirmBulkDelete}
  description={`Видалити ${tableActions.bulkDeleteIds.length} ${pluralizeUk(tableActions.bulkDeleteIds.length, ["працівника", "працівників", "працівників"])}? Цю дію неможливо відмінити.`}
/>
