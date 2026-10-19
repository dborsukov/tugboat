<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
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

  import { columns, type ServiceTableMeta } from "./columns";
  import {
    createArchiveServiceMutation,
    createArchiveServicesMutation,
    createDeleteServiceMutation,
    createDeleteServicesMutation,
    createRestoreServiceMutation,
    createRestoreServicesMutation,
    createServicesQuery,
  } from "./queries";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");
  let showArchived = $state(false);

  const queryClient = useQueryClient();

  const deleteServiceMutation = createDeleteServiceMutation(queryClient);
  const deleteServicesMutation = createDeleteServicesMutation(queryClient);
  const archiveServiceMutation = createArchiveServiceMutation(queryClient);
  const archiveServicesMutation = createArchiveServicesMutation(queryClient);
  const restoreServiceMutation = createRestoreServiceMutation(queryClient);
  const restoreServicesMutation = createRestoreServicesMutation(queryClient);

  const servicesQuery = createServicesQuery();

  const tableActions = new TableActionState<number>();

  let visibleServices = $derived.by(() => {
    const services = servicesQuery.isSuccess ? servicesQuery.data : [];
    return showArchived
      ? services.filter((service) => service.archivedAt != null)
      : services.filter((service) => service.archivedAt == null);
  });

  const archivedServicesCount = $derived(
    (servicesQuery.data ?? []).filter((service) => service.archivedAt != null)
      .length,
  );

  let table = buildTable({
    get data() {
      return visibleServices;
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
        onArchive: tableActions.openArchive,
        onRestore: (id: number) => restoreServiceMutation.mutate(id),
        onDelete: tableActions.openDelete,
      } satisfies ServiceTableMeta;
    },
    getRowId: (row) => String(row.id),
    initialSorting: [{ id: "name", desc: false }],
  });

  const selectedServiceIds = $derived(
    table.getFilteredSelectedRowModel().rows.map((row) => row.original.id),
  );

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

  function confirmArchive() {
    if (tableActions.archiveId !== null) {
      const id = tableActions.archiveId;
      archiveServiceMutation.mutate(id, {
        onSuccess: () => {
          tableActions.resetArchive();
          table.setRowSelection((prev) => {
            const next = { ...prev };
            delete next[String(id)];
            return next;
          });
        },
      });
    }
  }

  function confirmBulkArchive() {
    if (tableActions.bulkArchiveIds.length > 0) {
      archiveServicesMutation.mutate([...tableActions.bulkArchiveIds], {
        onSuccess: () => {
          tableActions.resetBulkArchive();
          table.resetRowSelection();
        },
      });
    }
  }

  function confirmBulkRestore() {
    if (selectedServiceIds.length > 0) {
      restoreServicesMutation.mutate([...selectedServiceIds], {
        onSuccess: () => {
          table.resetRowSelection();
        },
      });
    }
  }

  function handleToggleArchived() {
    showArchived = !showArchived;
    table.resetRowSelection();
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
        variant={showArchived ? "default" : "outline"}
        disabled={!servicesQuery.isSuccess}
        onclick={handleToggleArchived}
        class="shrink-0"
      >
        <ArchiveIcon />
        Архів
        {#if archivedServicesCount > 0}
          <span class="text-muted-foreground">{archivedServicesCount}</span>
        {/if}
      </Button>
      {#if !showArchived}
        <Button
          variant="outline"
          disabled={!servicesQuery.isSuccess}
          onclick={tableActions.openCreate}
        >
          Додати <PlusIcon />
        </Button>
      {/if}
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
      emptyMessage={showArchived
        ? "Архівних сервісів не знайдено..."
        : "Сервісів не знайдено..."}
      headerClasses={{ name: "w-full", select: "w-10 p-0" }}
      cellClasses={{ actions: "p-0", select: "p-0" }}
      rowClassName={(row) => (row.archivedAt ? "opacity-60" : undefined)}
      autoPageSize
      class="mt-4 rounded shadow-xs bg-card"
    >
      {#snippet selectionActions()}
        {#if showArchived}
          <DataTableBulkActions
            onDelete={() => tableActions.openBulkDelete(selectedServiceIds)}
            deleteLabel="Видалити назавжди"
          >
            {#snippet menuItems()}
              <DropdownMenu.Item onclick={confirmBulkRestore}>
                <ArchiveRestoreIcon />
                Відновити
              </DropdownMenu.Item>
            {/snippet}
          </DataTableBulkActions>
        {:else}
          <DataTableBulkActions
            onDelete={() => tableActions.openBulkArchive(selectedServiceIds)}
            deleteLabel="Архівувати"
            deleteVariant="warning"
            deleteIcon={ArchiveIcon}
          />
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</PageLayout>

<ServiceMutationSheet
  bind:open={tableActions.isMutationOpen}
  serviceId={tableActions.editId}
/>
<DeleteDialog
  bind:open={tableActions.isArchiveOpen}
  onDelete={confirmArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description="Сервіс буде прихований зі списку. Ви зможете відновити його пізніше."
/>
<DeleteDialog
  bind:open={tableActions.isBulkArchiveOpen}
  onDelete={confirmBulkArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description={`Архівувати ${tableActions.bulkArchiveIds.length} ${pluralizeUk(tableActions.bulkArchiveIds.length, ["сервіс", "сервіси", "сервісів"])}? Ви зможете відновити їх пізніше.`}
/>
<DeleteDialog
  bind:open={tableActions.isDeleteOpen}
  onDelete={confirmDelete}
  title="Видалити назавжди"
  description="Цю дію неможливо відмінити. Сервіс буде видалено безповоротно."
/>
<DeleteDialog
  bind:open={tableActions.isBulkDeleteOpen}
  onDelete={confirmBulkDelete}
  title="Видалити назавжди"
  description={`Видалити ${tableActions.bulkDeleteIds.length} ${pluralizeUk(tableActions.bulkDeleteIds.length, ["сервіс", "сервіси", "сервісів"])}? Цю дію неможливо відмінити.`}
/>
