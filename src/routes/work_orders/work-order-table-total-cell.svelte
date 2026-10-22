<script lang="ts">
  import Big from "big.js";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";

  import type { WorkOrderDetailsDto } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    workOrderDetails,
    class: className,
  }: {
    workOrderDetails: WorkOrderDetailsDto;
    class?: string;
  } = $props();

  let totalWithVat = $derived(
    new Big(workOrderDetails?.totals?.totalWithVat ?? "0"),
  );
  let belowCost = $derived(workOrderDetails?.totals?.belowCost ?? false);
</script>

<div
  class={cn(
    "flex w-full items-center justify-end gap-1.5 font-semibold tabular-nums",
    totalWithVat.eq(0) && "text-muted-foreground",
    belowCost && "text-rose-600 dark:text-rose-400",
    className,
  )}
  title={belowCost ? "Нижче собівартості" : undefined}
>
  <CurrencyDisplay amount={totalWithVat.toString()} suffix="грн." />
</div>
