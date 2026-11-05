import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type Report,
  type ReportKind,
  type VatPeriod,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
}

export function reportsQueryOptions() {
  return {
    queryKey: keys.reports.list(),
    queryFn: () => unwrap(commands.fetchReports()),
  };
}

export function createReportsQuery() {
  return createQuery<Report[]>(() => reportsQueryOptions());
}

export function createGenerateReportMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["reports", "generateReport"],
      mutationFn: (variables: {
        reportType: ReportKind;
        periodStart: string;
        periodEnd: string;
      }) =>
        unwrap(
          commands.generateReport(
            variables.reportType,
            variables.periodStart,
            variables.periodEnd,
          ),
        ),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: keys.reports.list() });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDownloadReportMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["reports", "downloadReport"],
      mutationFn: (variables: { id: number; fileName: string }) =>
        unwrap(commands.downloadReport(variables.id, variables.fileName)),
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createOpenReportCopyMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["reports", "openReportCopy"],
      mutationFn: (path: string) => unwrap(commands.openReportCopy(path)),
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

// ---------------------------------------------------------------------------
// VAT filing periods
// ---------------------------------------------------------------------------

const VAT_PERIODS_KEY = ["vat-periods"] as const;

export function vatPeriodsQueryOptions() {
  return {
    queryKey: VAT_PERIODS_KEY,
    queryFn: () => unwrap(commands.fetchVatPeriods()),
  };
}

export function createVatPeriodsQuery() {
  return createQuery<VatPeriod[]>(() => vatPeriodsQueryOptions());
}

export function createCloseVatPeriodMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["reports", "closeVatPeriod"],
      mutationFn: (variables: { month: string; note: string | null }) =>
        unwrap(commands.closeVatPeriod(variables.month, variables.note)),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: VAT_PERIODS_KEY });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createReopenVatPeriodMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["reports", "reopenVatPeriod"],
      mutationFn: (month: string) => unwrap(commands.reopenVatPeriod(month)),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: VAT_PERIODS_KEY });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}
