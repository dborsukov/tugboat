<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import { pluralizeUk } from "$lib/tools";

  import Money from "../components/money.svelte";
  import QueryError from "../components/query-error.svelte";
  import { createOutstandingBalanceQuery } from "../queries";
  import type { DashboardModuleProps } from "../registry";

  const query = createOutstandingBalanceQuery();

  let { slot }: DashboardModuleProps = $props();

  const orderCount = $derived(query.data?.orderCount ?? 0);
</script>

<Card class="h-full py-4">
  {#if query.isPending}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <Skeleton class="h-6 w-28" />
      <Skeleton class="h-5 w-32" />
      <div class="mt-auto">
        <Skeleton class="h-5 w-24" />
      </div>
    </CardContent>
  {:else if query.error}
    <CardContent
      class="flex min-h-0 flex-1 flex-col items-center justify-center"
    >
      <QueryError error={query.error} onRetry={() => query.refetch()} />
    </CardContent>
  {:else}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">
        <Money amount={query.data?.total ?? "0"} />
      </div>
      <p class="text-muted-foreground text-sm">Очікує на оплату</p>
      <div class="mt-auto text-sm">
        {#if orderCount === 0}
          <p class="text-muted-foreground/60">Немає неоплачених замовлень</p>
        {:else}
          <p class="text-muted-foreground">
            {orderCount}
            {#if slot.width === "sixth"}
              {pluralizeUk(orderCount, [
                "замовлення",
                "замовлення",
                "замовлень",
              ])}
            {:else}
              {pluralizeUk(orderCount, [
                "замовлення очікує",
                "замовлення очікує",
                "замовлень очікують",
              ])}
              оплати
            {/if}
          </p>
        {/if}
      </div>
    </CardContent>
  {/if}
</Card>
