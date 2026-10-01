<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import { pluralizeUk } from "$lib/tools";
  import { cn } from "$lib/utils";

  import ClientAvatar from "../../clients/client-avatar.svelte";
  import Meter from "../components/meter.svelte";
  import Money from "../components/money.svelte";
  import QueryError from "../components/query-error.svelte";
  import TrendLine from "../components/trend-line.svelte";
  import {
    dashboardHeightRows,
    dashboardWidthColumns,
  } from "../dashboard-grid";
  import { formatPercent, toNumber } from "../format";
  import { createClientMixListQuery, createClientMixQuery } from "../queries";
  import type { DashboardModuleProps, DashboardSlot } from "../registry";

  /** Module render sizes */
  type ContentSize = "total" | "mix" | "medium" | "large";

  /** Determines render size based on certain thresholds. */
  function contentSize(slot: DashboardSlot): ContentSize {
    const columns = dashboardWidthColumns[slot.width];
    const rows = dashboardHeightRows[slot.height];

    if (rows < dashboardHeightRows.two)
      return columns >= dashboardWidthColumns.third ? "mix" : "total";

    return columns >= dashboardWidthColumns.half ? "large" : "medium";
  }

  /** How many clients the list has room for. */
  function listRows(slot: DashboardSlot): number {
    return dashboardHeightRows[slot.height] >= dashboardHeightRows.three
      ? 6
      : 3;
  }

  let { slot }: DashboardModuleProps = $props();

  const query = createClientMixQuery();

  const size = $derived(contentSize(slot));
  const rows = $derived(listRows(slot));
  const lists = $derived(size === "medium" || size === "large");
  const listQuery = createClientMixListQuery(() => lists);

  const mix = $derived(query.data ?? null);
  const list = $derived(listQuery.data ?? null);

  const total = $derived(mix?.totalClients ?? 0);
  const newClients = $derived(mix?.newClients ?? 0);
  const returningClients = $derived(mix?.returningClients ?? 0);
  const previousTotal = $derived(mix?.previousTotalClients ?? 0);

  /** Clients more than last month: a count, not a share — one more is one more. */
  const clientChange = $derived(total - previousTotal);

  const clients = $derived(list?.clients ?? []);
  const averageCheck = $derived(toNumber(list?.averageCheck));

  /** Nothing has been billed this month yet. */
  const empty = $derived(total === 0);

  const groups = $derived([
    {
      key: "new",
      title: "Нові клієнти",
      color: "bg-chart-2",
      count: newClients,
      share: total === 0 ? 0 : (newClients / total) * 100,
      plural: ["новий", "нові", "нових"] as [string, string, string],
    },
    {
      key: "returning",
      title: "Постійні клієнти",
      color: "bg-muted-foreground/30",
      count: returningClients,
      share: total === 0 ? 0 : (returningClients / total) * 100,
      plural: ["постійний", "постійні", "постійних"] as [
        string,
        string,
        string,
      ],
    },
  ]);

  /** Count of returning customers. */
  const returningShare = $derived(groups[1].share);

  function countChange(value: number): string {
    return String(value);
  }

  function carLine(client: (typeof clients)[number]): string {
    return client.otherCars > 0
      ? `${client.car} +${client.otherCars}`
      : client.car;
  }
</script>

{#snippet clientBar()}
  <Meter
    class={cn(
      "h-2",
      empty && "border-border border border-dashed bg-transparent",
    )}
    segments={groups.map((group) => ({
      share: group.share,
      class: group.color,
      label: group.title,
    }))}
  />
{/snippet}

{#snippet clientLegend()}
  <div class="flex items-center justify-between gap-3 text-xs">
    {#each groups as group (group.key)}
      <span class="flex min-w-0 items-center gap-1.5" title={group.title}>
        <span
          class="size-2 shrink-0 rounded-full {group.color}"
          aria-hidden="true"
        ></span>
        <span class="font-medium">{group.count}</span>
        <span class="text-muted-foreground truncate">
          {pluralizeUk(group.count, group.plural)}
        </span>
      </span>
    {/each}
  </div>
{/snippet}

{#snippet clientMix()}
  <div class="flex items-center gap-4">
    <span class="text-3xl mb-1 font-bold tracking-tight">{total}</span>
    <div class="flex min-w-0 flex-1 flex-col gap-1.5">
      {@render clientBar()}
      {@render clientLegend()}
    </div>
  </div>
{/snippet}

{#snippet clientList(limit: number)}
  <div class="flex min-h-0 flex-1 flex-col">
    <div
      class="text-muted-foreground flex items-center gap-2 border-b pb-1 text-xs leading-tight"
    >
      <span class="min-w-0 flex-1">Клієнт</span>
      <span class="w-10 shrink-0 text-right">Візитів</span>
      <span class="w-24 shrink-0 text-right">Витрачено</span>
      <span class="w-16 shrink-0">Статус</span>
    </div>

    {#if listQuery.isPending}
      <div class="flex flex-col gap-3 pt-3">
        {#each Array(limit) as _, index (index)}
          <Skeleton class="h-5 w-full" />
        {/each}
      </div>
    {:else if listQuery.error}
      <QueryError
        error={listQuery.error}
        onRetry={() => listQuery.refetch()}
        class="pt-2"
      />
    {:else if clients.length === 0}
      <p class="text-muted-foreground pt-2 text-sm">
        Цього місяця ще немає клієнтів.
      </p>
    {:else}
      {#each clients.slice(0, limit) as client (client.id)}
        <div
          class="flex items-center gap-2 border-b border-border/60 py-1.5 last:border-b-0"
        >
          <ClientAvatar name={client.name} class="size-7" />
          <div class="flex min-w-0 flex-1 flex-col">
            <span class="truncate text-sm leading-tight" title={client.name}>
              {client.name}
            </span>
            <span
              class="text-muted-foreground truncate text-xs leading-tight"
              title={carLine(client)}
            >
              {carLine(client)}
            </span>
          </div>
          <span class="w-10 shrink-0 text-right text-sm tabular-nums">
            {client.visits}
          </span>
          <span class="w-24 shrink-0 text-right">
            <Money
              amount={client.spent}
              precision={0}
              class="justify-end gap-1"
            />
          </span>
          <span class="flex w-16 shrink-0 items-center">
            {#if client.isNew}
              <Badge class="bg-accent text-accent-foreground">новий</Badge>
            {:else}
              <span class="text-muted-foreground text-xs">постійний</span>
            {/if}
          </span>
        </div>
      {/each}
    {/if}
  </div>
{/snippet}

<Card class="h-full py-4">
  {#if query.isPending}
    {#if size === "total"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-6 w-12" />
        <Skeleton class="h-5 w-36" />
        <div class="mt-auto">
          <Skeleton class="h-5 w-32" />
        </div>
      </CardContent>
    {:else if size === "mix"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-2">
        <Skeleton class="h-5 w-36" />
        <div class="mt-auto flex flex-col gap-2">
          <Skeleton class="h-7 w-10" />
          <Skeleton class="h-2 w-full" />
          <Skeleton class="h-4 w-full" />
        </div>
      </CardContent>
    {:else}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
        <div class="flex items-start justify-between gap-4">
          <Skeleton class="h-5 w-36" />
          <Skeleton class="h-5 w-40" />
        </div>
        <div class="flex min-h-0 flex-1 gap-5">
          <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-3">
            <div class="flex items-center gap-4">
              <Skeleton class="h-7 w-10" />
              <div class="flex min-w-0 flex-1 flex-col gap-1.5">
                <Skeleton class="h-2 w-full" />
                <Skeleton class="h-4 w-full" />
              </div>
            </div>
            <div class="flex flex-col gap-3">
              {#each Array(size === "large" ? 3 : 4) as _, index (index)}
                <Skeleton class="h-6 w-full" />
              {/each}
            </div>
          </div>
          {#if size === "large"}
            <div class="flex w-[30%] shrink-0 flex-col gap-1.5">
              <Skeleton class="h-12" />
              <Skeleton class="h-12" />
            </div>
          {/if}
        </div>
      </CardContent>
    {/if}
  {:else if query.error}
    <CardContent
      class="flex min-h-0 flex-1 flex-col items-center justify-center"
    >
      <QueryError error={query.error} onRetry={() => query.refetch()} />
    </CardContent>
  {:else if size === "total"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">{total}</div>
      <p class="text-muted-foreground text-sm">Клієнти цього місяця</p>
      <div class="mt-auto text-sm">
        {#if empty}
          <p class="text-muted-foreground/60">Ще немає замовлень</p>
        {:else if newClients === 0}
          <p class="text-muted-foreground/60">Немає нових клієнтів</p>
        {:else}
          <p class="text-muted-foreground">
            {newClients}
            {pluralizeUk(newClients, [
              "новий клієнт",
              "нові клієнти",
              "нових клієнтів",
            ])}
          </p>
        {/if}
      </div>
    </CardContent>
  {:else if size === "mix"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-2">
      <p class="text-muted-foreground text-sm font-medium">
        Клієнти цього місяця
      </p>

      {#if empty}
        <p class="text-muted-foreground mt-auto text-sm">
          Цього місяця ще немає замовлень.
        </p>
      {:else}
        <div class="mt-auto flex flex-col gap-2">
          <div class="text-3xl font-bold tracking-tight">{total}</div>
          {@render clientBar()}
          {@render clientLegend()}
        </div>
      {/if}
    </CardContent>
  {:else}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
      <div class="flex items-start justify-between gap-4">
        <p class="text-muted-foreground text-sm font-medium">
          Клієнти цього місяця
        </p>
        {#if !empty}
          <div class="text-sm">
            <TrendLine
              change={clientChange}
              format={countChange}
              suffix="від минулого місяця"
            />
          </div>
        {/if}
      </div>

      {#if empty}
        <p class="text-muted-foreground text-sm">
          Цього місяця ще немає замовлень.
        </p>
      {:else if size === "medium"}
        {@render clientMix()}
        {@render clientList(rows)}
      {:else}
        <div class="flex min-h-0 flex-1 gap-5">
          <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-3">
            {@render clientMix()}
            {@render clientList(rows)}
          </div>
          <div class="flex w-[30%] shrink-0 flex-col gap-1.5">
            <div class="flex flex-col rounded-md px-2.5 py-1.5">
              <p class="text-muted-foreground text-xs leading-tight">
                Частка постійних
              </p>
              <p class="text-base font-semibold">
                {formatPercent(returningShare)}
              </p>
            </div>
            <div class="flex flex-col rounded-md px-2.5 py-1.5">
              <p class="text-muted-foreground text-xs leading-tight">
                Середній чек на клієнта
              </p>
              <div class="text-base font-semibold">
                {#if listQuery.error}
                  <span
                    class="text-muted-foreground"
                    aria-label="Немає даних"
                    title="Не вдалося завантажити">—</span
                  >
                {:else if listQuery.isPending}
                  <Skeleton class="h-5 w-20" />
                {:else}
                  <Money amount={averageCheck} precision={0} />
                {/if}
              </div>
            </div>
          </div>
        </div>
      {/if}
    </CardContent>
  {/if}
</Card>
