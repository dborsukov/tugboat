<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import FunnelXIcon from "@lucide/svelte/icons/funnel-x";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Input } from "$lib/components/ui/input";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import DataTableFacetedFilter from "$lib/components/data-table-faceted-filter/data-table-faceted-filter.svelte";
  import type { FacetedFilterOptions } from "$lib/components/data-table-faceted-filter/faceted-filter-options";
  import DataTableBulkActions from "$lib/components/data-table/data-table-bulk-actions.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { pluralizeUk } from "$lib/tools";

  import { expenseCategoryDotClass } from "./categories";
  import {
    CATEGORY_FILTER_COLUMN_ID,
    columns,
    type ExpenseTableMeta,
  } from "./columns";
  import ExpenseMutationSheet from "./expense-mutation-sheet.svelte";
  import {
    createArchiveExpenseMutation,
    createArchiveExpensesMutation,
    createDeleteExpenseMutation,
    createDeleteExpensesMutation,
    createExpenseCategoriesQuery,
    createExpensesQuery,
    createRestoreExpenseMutation,
    createRestoreExpensesMutation,
  } from "./queries";

  let inputValue: string = $state("");
  let showArchived = $state(false);

  const queryClient = useQueryClient();

  const deleteExpenseMutation = createDeleteExpenseMutation(queryClient);
  const deleteExpensesMutation = createDeleteExpensesMutation(queryClient);
  const archiveExpenseMutation = createArchiveExpenseMutation(queryClient);
  const archiveExpensesMutation = createArchiveExpensesMutation(queryClient);
  const restoreExpenseMutation = createRestoreExpenseMutation(queryClient);
  const restoreExpensesMutation = createRestoreExpensesMutation(queryClient);

  const expensesQuery = createExpensesQuery();
  const categoriesQuery = createExpenseCategoriesQuery();

  const tableActions = new TableActionState<number>();

  const categories = $derived(categoriesQuery.data ?? []);

  let visibleExpenses = $derived.by(() => {
    const expenses = expensesQuery.isSuccess ? expensesQuery.data : [];
    return showArchived
      ? expenses.filter((expense) => expense.archivedAt != null)
      : expenses.filter((expense) => expense.archivedAt == null);
  });

  const archivedExpensesCount = $derived(
    (expensesQuery.data ?? []).filter((expense) => expense.archivedAt != null)
      .length,
  );

  let table = buildTable({
    get data() {
      return visibleExpenses;
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
        onRestore: (id: number) => restoreExpenseMutation.mutate(id),
        onDelete: tableActions.openDelete,
      } satisfies ExpenseTableMeta;
    },
    getRowId: (row) => String(row.id),
    hiddenColumns: [CATEGORY_FILTER_COLUMN_ID],
    initialSorting: [{ id: "date", desc: true }],
  });

  const categoryColumn = $derived(table.getColumn("category"));
  const categoryIdColumn = $derived(table.getColumn(CATEGORY_FILTER_COLUMN_ID));

  /** Ids the category filter currently holds, as the strings it stores them as. */
  const categoryFilterIds = $derived(
    (categoryIdColumn?.getFilterValue() as string[] | undefined) ?? [],
  );

  const categoryFilterOptions = $derived.by<FacetedFilterOptions[]>(() => {
    const present = new Set(
      visibleExpenses
        .map((expense) => expense.category?.id)
        .filter((id): id is number => id != null),
    );
    // A category whose expenses are all filtered away would otherwise vanish
    // from the picker and leave the filter impossible to clear from it.
    for (const value of categoryFilterIds) present.add(Number(value));

    return categories
      .filter((category) => present.has(category.id))
      .sort((a, b) => a.name.localeCompare(b.name, "uk"))
      .map((category) => ({
        label: category.name,
        value: String(category.id),
        dotClass: expenseCategoryDotClass(category.color),
      }));
  });

  const isCategoryFiltered = $derived(categoryFilterIds.length > 0);

  const selectedExpenseIds = $derived(
    table.getFilteredSelectedRowModel().rows.map((row) => row.original.id),
  );

  function confirmDelete() {
    if (tableActions.deleteId !== null) {
      const id = tableActions.deleteId;
      deleteExpenseMutation.mutate(id, {
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
      deleteExpensesMutation.mutate([...tableActions.bulkDeleteIds], {
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
      archiveExpenseMutation.mutate(id, {
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
      archiveExpensesMutation.mutate([...tableActions.bulkArchiveIds], {
        onSuccess: () => {
          tableActions.resetBulkArchive();
          table.resetRowSelection();
        },
      });
    }
  }

  function confirmBulkRestore() {
    if (selectedExpenseIds.length > 0) {
      restoreExpensesMutation.mutate([...selectedExpenseIds], {
        onSuccess: () => {
          table.resetRowSelection();
        },
      });
    }
  }

  function handleToggleArchived() {
    showArchived = !showArchived;
    table.resetColumnFilters();
    table.resetRowSelection();
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full flex items-center gap-2">
      <Input
        placeholder="Пошук за описом, категорією, сумою або датою..."
        disabled={!expensesQuery.isSuccess}
        bind:value={inputValue}
        class="w-full"
      />
      {#if categoryColumn && categoryIdColumn}
        <DataTableFacetedFilter
          column={categoryColumn}
          filterColumn={categoryIdColumn}
          title="Категорія"
          options={categoryFilterOptions}
          disabled={!expensesQuery.isSuccess}
        />
      {/if}
      {#if isCategoryFiltered}
        <Button
          size="default"
          variant="ghost"
          onclick={() => table.resetColumnFilters()}
          class="px-2 py-4"
          disabled={!expensesQuery.isSuccess}
        >
          <FunnelXIcon />
          Прибрати фільтри
        </Button>
      {/if}
      <Button
        variant={showArchived ? "default" : "outline"}
        disabled={!expensesQuery.isSuccess}
        onclick={handleToggleArchived}
        class="shrink-0"
      >
        <ArchiveIcon />
        Архів
        {#if archivedExpensesCount > 0}
          <span class="text-muted-foreground">{archivedExpensesCount}</span>
        {/if}
      </Button>
      {#if !showArchived}
        <Button
          disabled={!expensesQuery.isSuccess}
          onclick={tableActions.openCreate}
        >
          Додати <PlusIcon />
        </Button>
      {/if}
    </div>
  {/snippet}
  {#if expensesQuery.isError}
    <FetchErrorDisplay
      message={expensesQuery.error?.message}
      onRetry={expensesQuery.refetch}
      class="mt-10"
    />
  {:else if expensesQuery.isLoading}
    <Skeleton class="mt-4 h-50 w-full rounded-lg" />
  {:else}
    <DataTable
      {table}
      emptyMessage={showArchived
        ? "Архівних витрат не знайдено..."
        : "Витрат не знайдено..."}
      headerClasses={{
        select: "pl-4 pr-0",
        description: "w-full",
        amount: "text-right",
      }}
      cellClasses={{ select: "pl-4 pr-0", actions: "p-0.5" }}
      rowClassName={(row) => (row.archivedAt ? "opacity-60" : undefined)}
      autoPageSize
      class="mt-4"
    >
      {#snippet selectionActions()}
        {#if showArchived}
          <DataTableBulkActions
            onDelete={() => tableActions.openBulkDelete(selectedExpenseIds)}
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
            onDelete={() => tableActions.openBulkArchive(selectedExpenseIds)}
            deleteLabel="Архівувати"
            deleteVariant="warning"
            deleteIcon={ArchiveIcon}
          />
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</PageLayout>

<ExpenseMutationSheet
  bind:open={tableActions.isMutationOpen}
  expenseId={tableActions.editId}
/>
<DeleteDialog
  bind:open={tableActions.isArchiveOpen}
  onDelete={confirmArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description="Витрата буде прихована зі списку. Ви зможете відновити її пізніше."
/>
<DeleteDialog
  bind:open={tableActions.isBulkArchiveOpen}
  onDelete={confirmBulkArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description={`Архівувати ${tableActions.bulkArchiveIds.length} ${pluralizeUk(tableActions.bulkArchiveIds.length, ["витрату", "витрати", "витрат"])}? Ви зможете відновити їх пізніше.`}
/>
<DeleteDialog
  bind:open={tableActions.isDeleteOpen}
  onDelete={confirmDelete}
  title="Видалити назавжди"
  description="Цю дію неможливо відмінити. Витрату буде видалено безповоротно."
/>
<DeleteDialog
  bind:open={tableActions.isBulkDeleteOpen}
  onDelete={confirmBulkDelete}
  title="Видалити назавжди"
  description={`Видалити ${tableActions.bulkDeleteIds.length} ${pluralizeUk(tableActions.bulkDeleteIds.length, ["витрату", "витрати", "витрат"])}? Цю дію неможливо відмінити.`}
/>
