<script lang="ts">
  import Big from "big.js";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ReceiptIcon from "@lucide/svelte/icons/receipt-text";

  import { buttonVariants } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";

  import { type WorkOrderDetailsDto } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    workOrderDetails,
    class: className,
  }: {
    workOrderDetails: WorkOrderDetailsDto;
    class?: string;
  } = $props();

  let laborDiscount = $derived(
    new Big(workOrderDetails?.totals?.laborDiscount ?? "0"),
  );
  let partsDiscount = $derived(
    new Big(workOrderDetails?.totals?.partsDiscount ?? "0"),
  );
  let orderDiscount = $derived(
    new Big(workOrderDetails?.totals?.orderDiscount ?? "0"),
  );
</script>

<Popover.Root>
  <Popover.Trigger
    class={cn(
      buttonVariants({ size: "lg", variant: "outline" }),
      "text-base px-2 border-none bg-green-100",
      className,
    )}
  >
    <ReceiptIcon class="size-5 text-green-700" />
    <CurrencyDisplay
      amount={workOrderDetails?.totals?.grandTotal ?? "0"}
      suffix="грн."
      class="text-base font-medium text-green-700"
    />
    <ChevronDownIcon class="size-5" />
  </Popover.Trigger>
  <Popover.Content align="end" class="w-72 p-4">
    <div class="flex flex-col gap-3">
      <div class="flex items-baseline justify-between">
        <span class="text-sm text-muted-foreground">Робота</span>
        <CurrencyDisplay
          amount={workOrderDetails?.totals?.laborTotal ?? "0"}
          suffix="грн."
          class="text-sm font-medium"
        />
      </div>
      {#if laborDiscount.gt(0)}
        <div class="flex flex-col gap-1 border-l border-border pl-3">
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Сума</span>
            <span
              class="flex items-baseline gap-0.5 text-xs text-muted-foreground"
            >
              <span></span>
              <CurrencyDisplay
                amount={workOrderDetails?.totals?.laborCost.toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Знижка</span>
            <span
              class="flex items-baseline gap-0.5 text-xs text-muted-foreground"
            >
              <span>-</span>
              <CurrencyDisplay
                amount={laborDiscount.toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
        </div>
      {/if}
      <div class="flex items-baseline justify-between">
        <span class="text-sm text-muted-foreground">Запчастини</span>
        <CurrencyDisplay
          amount={workOrderDetails?.totals?.partsTotal ?? "0"}
          suffix="грн."
          class="text-sm font-medium"
        />
      </div>
      {#if partsDiscount.gt(0)}
        <div class="flex flex-col gap-1 border-l border-border pl-3">
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Сума</span>
            <span
              class="flex items-baseline gap-0.5 text-xs text-muted-foreground"
            >
              <span></span>
              <CurrencyDisplay
                amount={Big(workOrderDetails?.totals?.partsTotal)
                  .plus(partsDiscount)
                  .toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Знижка</span>
            <span
              class="flex items-baseline gap-0.5 text-xs text-muted-foreground"
            >
              <span>-</span>
              <CurrencyDisplay
                amount={partsDiscount.toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
        </div>
      {/if}
      {#if orderDiscount.gt(0)}
        <div class="h-px bg-border"></div>
        <div class="flex items-baseline justify-between">
          <span class="text-sm text-muted-foreground">Проміжний підсумок</span>
          <CurrencyDisplay
            amount={workOrderDetails?.totals?.subtotal ?? "0"}
            suffix="грн."
            class="text-sm font-medium"
          />
        </div>
        <div class="flex flex-col gap-1 border-l border-border pl-3">
          <div class="flex items-baseline justify-between">
            <span class="flex gap-1 text-xs text-muted-foreground">
              <p>Знижка</p>
              {#if workOrderDetails.workOrder.discountType === "Percentage"}
                <span>({workOrderDetails.workOrder.discountPercent}%)</span>
              {:else}
                <span>(фіксована)</span>
              {/if}
            </span>
            <span
              class="flex items-baseline gap-0.5 text-xs text-muted-foreground"
            >
              <span>-</span>
              <CurrencyDisplay
                amount={orderDiscount.toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
        </div>
      {/if}
      <div class="h-px bg-border"></div>
      <div
        class="flex items-baseline justify-between rounded-md bg-green-100 px-3 py-2"
      >
        <span class="text-sm font-medium text-green-700">Всього</span>
        <CurrencyDisplay
          amount={workOrderDetails?.totals?.grandTotal ?? "0"}
          suffix="грн."
          class="text-lg font-medium text-green-700"
        />
      </div>
    </div>
  </Popover.Content>
</Popover.Root>
