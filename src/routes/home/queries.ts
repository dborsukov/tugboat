import { createQuery, type QueryClient } from "@tanstack/svelte-query";

import type {
  AnalyticsPeriod,
  ClientMix,
  ClientMixList,
  DashboardLayout,
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
import { createSettingsFieldMutation } from "$lib/states/settings.svelte";
import { unwrap } from "$lib/tools";

import { booleanSetting } from "./dashboard-layout";
import { timeZoneOffsetMinutes } from "./period";
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

export function createDashboardLayoutMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "dashboardLayout",
    (dashboardLayout: DashboardLayout) =>
      unwrap(commands.updateDashboardLayout(dashboardLayout)),
  );
}
