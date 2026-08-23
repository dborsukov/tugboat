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

  let paid = $derived(workOrderDetails?.workOrder.status === "Paid");
  let grandTotal = $derived(
    new Big(workOrderDetails?.totals?.grandTotal ?? "0"),
  );
  let partsCost = $derived(new Big(workOrderDetails?.totals?.partsCost ?? "0"));
  let laborCost = $derived(new Big(workOrderDetails?.totals?.laborCost ?? "0"));
  let totalCost = $derived(partsCost.plus(laborCost));
  let belowCost = $derived(grandTotal.lt(totalCost));
</script>

<div
  class={cn(
    "w-full h-full flex items-center justify-center",
    grandTotal.eq(0) && "text-muted-foreground",
    paid &&
      "bg-green-100 text-green-700 dark:bg-green-500/15 dark:text-green-300 font-medium",
    belowCost &&
      "bg-rose-100 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300 font-medium",
    className,
  )}
>
  <CurrencyDisplay amount={grandTotal.toString()} suffix="грн." />
</div>
