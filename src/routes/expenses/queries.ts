import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type CreateExpenseCategoryPayload,
  type CreateExpensePayload,
  type Expense,
  type ExpenseCategory,
  type UpdateExpenseCategoryPayload,
  type UpdateExpensePayload,
  type VatEntry,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
}

export function expensesCountQueryOptions() {
  return {
    queryKey: keys.expenses.count(),
    queryFn: () => unwrap(commands.fetchExpensesCount()),
  };
}

export function createExpensesCountQuery() {
  return createQuery<number>(() => expensesCountQueryOptions());
}

export function expensesQueryOptions() {
  return {
    queryKey: keys.expenses.list(),
    queryFn: () => unwrap(commands.fetchExpensesList()),
  };
}

export function createExpensesQuery() {
  return createQuery<Expense[]>(() => expensesQueryOptions());
}

export function expenseByIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.expenses.detail(id!),
    queryFn: () => unwrap(commands.fetchExpenseById(id!)),
  };
}

export function createExpenseByIdQuery(getId: () => number | null) {
  return createQuery<Expense>(() => expenseByIdQueryOptions(getId()));
}

export function createCreateExpenseMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: CreateExpensePayload) =>
        unwrap(commands.createExpense(payload)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateExpenseMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: { id: number; payload: UpdateExpensePayload }) =>
        unwrap(commands.updateExpense(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteExpenseMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deleteExpense(id)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteExpensesMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (ids: number[]) => unwrap(commands.deleteExpenses(ids)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveExpenseMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.archiveExpense(id)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveExpensesMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (ids: number[]) => unwrap(commands.archiveExpenses(ids)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreExpenseMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.restoreExpense(id)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreExpensesMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (ids: number[]) => unwrap(commands.restoreExpenses(ids)),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateExpenseLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.expenses.all });
  queryClient.invalidateQueries({ queryKey: keys.dashboard.all });
}

// ---------------------------------------------------------------------------
// Input VAT
// ---------------------------------------------------------------------------

export function expenseVatQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.expenses.vat(id!),
    queryFn: () => unwrap(commands.fetchExpenseVat(id!)),
  };
}

export function createExpenseVatQuery(getId: () => number | null) {
  return createQuery<VatEntry[]>(() => expenseVatQueryOptions(getId()));
}

/** The credit in force: neither a reversal nor a voided original. */
export function liveExpenseVatEntry(entries: VatEntry[] | undefined) {
  return entries?.find(
    (entry) => entry.reversesId == null && entry.voidedAt == null,
  );
}

export function createVoidExpenseVatMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: {
        id: number;
        creditNoteNumber: string | null;
        creditNoteDate: string | null;
      }) =>
        unwrap(
          commands.voidExpenseVat(
            variables.id,
            variables.creditNoteNumber,
            variables.creditNoteDate,
          ),
        ),
      onSuccess: () => {
        invalidateExpenseLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function expenseCategoriesQueryOptions() {
  return {
    queryKey: keys.expenseCategories.list(),
    queryFn: () => unwrap(commands.fetchExpenseCategories()),
  };
}

export function createExpenseCategoriesQuery() {
  return createQuery<ExpenseCategory[]>(() => expenseCategoriesQueryOptions());
}

export function createCreateExpenseCategoryMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: CreateExpenseCategoryPayload) =>
        unwrap(commands.createExpenseCategory(payload)),
      onSuccess: () => {
        invalidateExpenseCategoryLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateExpenseCategoryMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: {
        id: number;
        payload: UpdateExpenseCategoryPayload;
      }) =>
        unwrap(commands.updateExpenseCategory(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateExpenseCategoryLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveExpenseCategoryMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.archiveExpenseCategory(id)),
      onSuccess: () => {
        invalidateExpenseCategoryLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreExpenseCategoryMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.restoreExpenseCategory(id)),
      onSuccess: () => {
        invalidateExpenseCategoryLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateExpenseCategoryLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.expenseCategories.list() });
  queryClient.invalidateQueries({ queryKey: keys.dashboard.all });
}
