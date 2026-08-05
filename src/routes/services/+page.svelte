<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { commands } from "$lib/bindings";
  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { unwrap } from "$lib/tools";

  import { columns, type ServiceTableMeta } from "./columns";
  import { createServicesQuery, invalidateServiceLists } from "./queries";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");

  const queryClient = useQueryClient();
  const servicesQuery = createServicesQuery();

  const deleteMutation = createMutation(() => ({
    mutationFn: (id: number) => unwrap(commands.deleteService(id)),
    onSuccess: () => {
      invalidateServiceLists(queryClient);
      tableActions.resetDelete();
    },
    onError: (error) => {
      toast.error("Виникла помилка", { description: error.message });
    },
  }));

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
      deleteMutation.mutate(tableActions.deleteId);
    }
  }
</script>

<PageLayout>
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
        onclick={tableActions.openCreate}
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

<ServiceMutationSheet
  bind:open={tableActions.isMutationOpen}
  id={tableActions.editId}
/>
<DeleteDialog
  bind:open={tableActions.isDeleteOpen}
  onDelete={confirmDelete}
  description="Цю дію неможливо відмінити."
/>
