import { createQuery, type QueryClient } from "@tanstack/svelte-query";

import type {
  AnalyticsPeriod,
  ClientMix,
  ClientMixList,
  DashboardLayout,
  DaySpan,
  ExpenseSummary,
  InProgressBalance,
  MarginOptions,
  MarginService,
  MarginSummary,
  OrderMonth,
  OutstandingBalance,
  RevenueSummary,
  TopServicesReport,
  VatDue,
} from "$lib/bindings";
import { commands } from "$lib/bindings";
import { keys } from "$lib/query-keys";
import {
  createSettingsFieldMutation,
  settingsState,
} from "$lib/states/settings.svelte";
import { unwrap } from "$lib/tools";

import { booleanSetting } from "./dashboard-layout";
import { GROSS_INCOME_ID } from "./module-ids";
import {
  analyticsPeriod,
  calendarMonthOf,
  monthDays,
  monthName,
  previousMonth,
  timeZoneOffsetMinutes,
} from "./period";
import { dashboardPeriod } from "./state.svelte";

const STALE_TIME = 30_000;

function moduleOptions<T>(
  id: string,
  queryFn: () => Promise<T>,
  /** Anything the answer depends on */
  keyPart?: unknown,
) {
  return {
    queryKey: keys.dashboard.module(id, keyPart),
    queryFn,
    staleTime: STALE_TIME,
  };
}

function currentPeriod(): AnalyticsPeriod {
  return dashboardPeriod.period;
}

export function orderMonthsQueryOptions(utcOffsetMinutes: number) {
  return moduleOptions(
    "order-months",
    () => unwrap(commands.fetchOrderMonths(utcOffsetMinutes)),
    utcOffsetMinutes,
  );
}

export function createOrderMonthsQuery() {
  return createQuery<OrderMonth[]>(() =>
    orderMonthsQueryOptions(timeZoneOffsetMinutes(dashboardPeriod.timeZone)),
  );
}

export function outstandingBalanceQueryOptions() {
  return moduleOptions("outstanding-balance", () =>
    unwrap(commands.fetchOutstandingBalance()),
  );
}

export function createOutstandingBalanceQuery() {
  return createQuery<OutstandingBalance>(() =>
    outstandingBalanceQueryOptions(),
  );
}

export function inProgressBalanceQueryOptions() {
  return moduleOptions("in-progress-balance", () =>
    unwrap(commands.fetchInProgressBalance()),
  );
}

export function createInProgressBalanceQuery() {
  return createQuery<InProgressBalance>(() => inProgressBalanceQueryOptions());
}

export function vatDueQueryOptions(period: AnalyticsPeriod) {
  return moduleOptions(
    "vat-due",
    () => unwrap(commands.fetchVatDue(period)),
    period,
  );
}

export function createVatDueQuery() {
  return createQuery<VatDue>(() => vatDueQueryOptions(currentPeriod()));
}

export function revenueSummaryQueryOptions(period: AnalyticsPeriod) {
  return moduleOptions(
    "revenue-summary",
    () => unwrap(commands.fetchRevenueSummary(period)),
    period,
  );
}

export function createRevenueSummaryQuery() {
  return createQuery<RevenueSummary>(() =>
    revenueSummaryQueryOptions(currentPeriod()),
  );
}

export function topServicesQueryOptions(period: AnalyticsPeriod) {
  return moduleOptions(
    "top-services",
    () => unwrap(commands.fetchTopServices(period)),
    period,
  );
}

export function createTopServicesQuery() {
  return createQuery<TopServicesReport>(() =>
    topServicesQueryOptions(currentPeriod()),
  );
}

export function clientMixQueryOptions(period: AnalyticsPeriod) {
  return moduleOptions(
    "client-mix",
    () => unwrap(commands.fetchClientMix(period)),
    period,
  );
}

export function createClientMixQuery() {
  return createQuery<ClientMix>(() => clientMixQueryOptions(currentPeriod()));
}

export function clientMixListQueryOptions(period: AnalyticsPeriod) {
  return moduleOptions(
    "client-mix-list",
    () => unwrap(commands.fetchClientMixList(period)),
    period,
  );
}

export function createClientMixListQuery(enabled: () => boolean = () => true) {
  return createQuery<ClientMixList>(() => ({
    ...clientMixListQueryOptions(currentPeriod()),
    enabled: enabled(),
  }));
}

function marginOptions(id: string): MarginOptions {
  return {
    includeLaborCost: booleanSetting(id, "includeLaborCost"),
  };
}

export function grossMarginQueryOptions(
  period: AnalyticsPeriod,
  options: MarginOptions,
) {
  return moduleOptions(
    "gross-margin",
    () => unwrap(commands.fetchGrossMargin(period, options)),
    { period, options },
  );
}

export function createGrossMarginQuery(id: string) {
  return createQuery<MarginSummary>(() =>
    grossMarginQueryOptions(currentPeriod(), marginOptions(id)),
  );
}

export function marginServicesQueryOptions(
  period: AnalyticsPeriod,
  options: MarginOptions,
) {
  return moduleOptions(
    "margin-services",
    () => unwrap(commands.fetchMarginServices(period, options)),
    { period, options },
  );
}

export function createMarginServicesQuery(
  id: string,
  enabled: () => boolean = () => true,
) {
  return createQuery<MarginService[]>(() => ({
    ...marginServicesQueryOptions(currentPeriod(), marginOptions(id)),
    enabled: enabled(),
  }));
}

function currentExpenseMonths(): { month: DaySpan; previous: DaySpan } {
  const month = dashboardPeriod.anchor;

  return { month: monthDays(month), previous: monthDays(previousMonth(month)) };
}

export function expenseSummaryQueryOptions(month: DaySpan, previous: DaySpan) {
  return moduleOptions(
    "expense-summary",
    () => unwrap(commands.fetchExpenseSummary(month, previous)),
    { month, previous },
  );
}

export function createExpenseSummaryQuery() {
  return createQuery<ExpenseSummary>(() => {
    const { month, previous } = currentExpenseMonths();

    return expenseSummaryQueryOptions(month, previous);
  });
}

/** Current calendar month and the figures behind the sidebar's home line. */
export function currentMonthFinancialSummaryQueryOptions() {
  const now = new Date();
  const timeZone = settingsState.settings?.timezone ?? null;
  const anchor = calendarMonthOf(now, timeZone);
  const period = analyticsPeriod(now, timeZone, anchor);

  return {
    month: monthName(anchor, timeZone),
    queries: [
      grossMarginQueryOptions(period, marginOptions(GROSS_INCOME_ID)),
      expenseSummaryQueryOptions(
        monthDays(anchor),
        monthDays(previousMonth(anchor)),
      ),
      revenueSummaryQueryOptions(period),
    ],
  };
}

export function createDashboardLayoutMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "dashboardLayout",
    (dashboardLayout: DashboardLayout) =>
      unwrap(commands.updateDashboardLayout(dashboardLayout)),
  );
}
