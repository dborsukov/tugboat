<script lang="ts">
  import Big from "big.js";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

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
  let partsPendingTotal = $derived(
    new Big(workOrderDetails?.totals?.partsPendingTotal ?? "0"),
  );

  let freeLaborTotal = $derived(
    (workOrderDetails?.laborItems ?? []).reduce(
      (acc, item) => (item.isFree ? acc.plus(item.totals.total) : acc),
      new Big(0),
    ),
  );
  let freeLaborCost = $derived(
    (workOrderDetails?.laborItems ?? []).reduce(
      (acc, item) => (item.isFree ? acc.plus(item.totals.cost) : acc),
      new Big(0),
    ),
  );
  let freePartsTotal = $derived(
    (workOrderDetails?.partItems ?? []).reduce(
      (acc, item) => (item.isFree ? acc.plus(item.totals.total) : acc),
      new Big(0),
    ),
  );
  let freePartsCost = $derived(
    (workOrderDetails?.partItems ?? []).reduce(
      (acc, item) => (item.isFree ? acc.plus(item.totals.cost) : acc),
      new Big(0),
    ),
  );
  let orderDiscount = $derived(
    new Big(workOrderDetails?.totals?.orderDiscount ?? "0"),
  );
  let vatAmount = $derived(new Big(workOrderDetails?.totals?.vatAmount ?? "0"));
  let vatBase = $derived(new Big(workOrderDetails?.totals?.vatBase ?? "0"));
  let totalWithVat = $derived(
    new Big(workOrderDetails?.totals?.totalWithVat ?? "0"),
  );
  let vatRate = $derived(workOrderDetails?.workOrder?.vatRate ?? "0");
  let pricesIncludeVat = $derived(
    workOrderDetails?.workOrder?.pricesIncludeVat ?? false,
  );
  let hasVat = $derived(vatAmount.gt(0));
  let partsCost = $derived(new Big(workOrderDetails?.totals?.partsCost ?? "0"));
  let laborCost = $derived(new Big(workOrderDetails?.totals?.laborCost ?? "0"));
  let totalCost = $derived(partsCost.plus(laborCost));
  // Computed by the backend: money actually kept (net of output VAT) vs cost.
  let belowCost = $derived(workOrderDetails?.totals?.belowCost ?? false);
  let totalIsZero = $derived(totalWithVat.eq(0));
</script>

<Popover.Root>
  <Popover.Trigger
    class={cn(
      buttonVariants({ size: "lg", variant: "outline" }),
      "h-10 gap-2 rounded-4xl px-3.5 text-base font-medium",
      belowCost &&
        "border-rose-300 bg-rose-100 text-rose-800 hover:bg-rose-200 hover:text-rose-900 aria-expanded:bg-rose-100 aria-expanded:text-rose-800 dark:border-rose-500/30 dark:bg-rose-500/15 dark:text-rose-300 dark:hover:bg-rose-500/25 dark:hover:text-rose-200 dark:aria-expanded:bg-rose-500/15 dark:aria-expanded:text-rose-300",
      className,
    )}
  >
    <CurrencyDisplay
      amount={totalWithVat.toString()}
      suffix="грн."
      class={cn("text-base font-semibold")}
    />
    <ChevronDownIcon
      class={cn(
        "size-4",
        belowCost
          ? "text-rose-700 dark:text-rose-300"
          : "text-muted-foreground",
      )}
    />
  </Popover.Trigger>
  <Popover.Content align="end" class="w-72 p-4">
    <div class="flex flex-col gap-3">
      <div class="flex items-baseline justify-between">
        <span class="text-sm">Робота</span>
        <CurrencyDisplay
          amount={workOrderDetails?.totals?.laborTotal ?? "0"}
          suffix="грн."
          class="text-sm font-medium"
        />
      </div>
      {#if laborDiscount.gt(0)}
        <div class="flex flex-col gap-1 border-l border-border pl-3">
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Сума без знижки</span>
            <span class="flex items-baseline text-xs text-muted-foreground">
              <span></span>
              <CurrencyDisplay
                amount={Big(workOrderDetails?.totals.laborTotal)
                  .plus(laborDiscount)
                  .toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Знижка</span>
            <span class="flex items-baseline text-xs text-muted-foreground">
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
      {#if freeLaborTotal.gt(0)}
        <div
          class="flex flex-col gap-1 rounded-md border border-border bg-muted/40 px-3 py-1.5"
        >
          <p class="text-xs text-muted-foreground">
            Деякі роботи були надані безкоштовно
          </p>
          {#if freeLaborCost.gt(0)}
            <div class="flex items-baseline justify-between">
              <span class="text-xs text-muted-foreground"
                >Їхня собівартість:</span
              >
              <CurrencyDisplay
                amount={freeLaborCost.toString()}
                suffix="грн."
                class="text-xs font-medium text-foreground"
              />
            </div>
          {/if}
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Їхня вартість:</span>
            <CurrencyDisplay
              amount={freeLaborTotal.toString()}
              suffix="грн."
              class="text-xs font-medium text-foreground"
            />
          </div>
        </div>
      {/if}
      <div class="flex items-baseline justify-between">
        <span class="text-sm">Запчастини</span>
        <CurrencyDisplay
          amount={workOrderDetails?.totals?.partsTotal ?? "0"}
          suffix="грн."
          class="text-sm font-medium"
        />
      </div>
      {#if partsDiscount.gt(0)}
        <div class="flex flex-col gap-1 border-l border-border pl-3">
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Сума без знижки</span>
            <span class="flex items-baseline text-xs text-muted-foreground">
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
            <span class="flex items-baseline text-xs text-muted-foreground">
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
      {#if freePartsTotal.gt(0)}
        <div
          class="flex flex-col gap-1 rounded-md border border-border bg-muted/40 px-3 py-1.5"
        >
          <p class="text-xs text-muted-foreground">
            Деякі запчастини були надані безкоштовно
          </p>
          {#if freePartsCost.gt(0)}
            <div class="flex items-baseline justify-between">
              <span class="text-xs text-muted-foreground"
                >Їхня собівартість:</span
              >
              <CurrencyDisplay
                amount={freePartsCost.toString()}
                suffix="грн."
                class="text-xs font-medium text-foreground"
              />
            </div>
          {/if}
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Їхня вартість:</span>
            <CurrencyDisplay
              amount={freePartsTotal.toString()}
              suffix="грн."
              class="text-xs font-medium text-foreground"
            />
          </div>
        </div>
      {/if}
      {#if partsPendingTotal.gt(0)}
        <div
          class="flex flex-col gap-1 rounded-md border border-border bg-muted/40 px-3 py-1.5"
        >
          <div class="flex gap-1 items-center justify-between">
            <span class="text-xs text-muted-foreground">
              Деякі запчастини очікують встановлення, їхня вартість не
              враховується та становить:</span
            >
            <CurrencyDisplay
              amount={partsPendingTotal.toString()}
              suffix="грн."
              class="text-xs font-medium text-foreground"
            />
          </div>
        </div>
      {/if}
      <div class="h-px bg-border"></div>
      <div class="flex items-baseline justify-between">
        <span class="text-sm">Разом</span>
        <CurrencyDisplay
          amount={workOrderDetails?.totals?.subtotal ?? "0"}
          suffix="грн."
          class="text-sm font-medium"
        />
      </div>
      <div class="h-px bg-border"></div>
      {#if orderDiscount.gt(0)}
        <div class="flex items-baseline justify-between text-sm">
          <span class="flex gap-1">
            <p>Знижка</p>
            {#if workOrderDetails.workOrder.discountType === "Percentage"}
              <span>({workOrderDetails.workOrder.discountValue}%)</span>
            {:else}
              <span>(фіксована)</span>
            {/if}
          </span>
          <span class="flex items-baseline">
            <span>-</span>
            <CurrencyDisplay amount={orderDiscount.toString()} suffix="грн." />
          </span>
        </div>
      {/if}
      {#if hasVat && !pricesIncludeVat}
        {#if orderDiscount.gt(0)}
          <div class="flex items-baseline justify-between text-sm">
            <span>Зі знижкою</span>
            <CurrencyDisplay amount={vatBase.toString()} suffix="грн." />
          </div>
        {/if}
        {#if !pricesIncludeVat}
          <div class="flex items-baseline justify-between text-sm">
            <span>
              ПДВ ({vatRate}%)
            </span>
            <span class="flex items-baseline">
              <span>+</span>
              <CurrencyDisplay amount={vatAmount.toString()} suffix="грн." />
            </span>
          </div>
        {/if}
      {:else if vatRate === "0"}
        <div
          class="flex items-baseline justify-between text-sm text-muted-foreground"
        >
          <span>ПДВ</span>
          <span>без ПДВ</span>
        </div>
      {/if}
      <div
        class={cn(
          "flex items-baseline justify-between rounded-md px-3 py-2",
          !totalIsZero && "bg-green-100 dark:bg-green-500/15",
          totalIsZero && !belowCost && "bg-muted",
          belowCost && "bg-rose-100 dark:bg-rose-500/15",
        )}
      >
        <span
          class={cn(
            "text-sm font-medium",
            !totalIsZero && "text-green-700 dark:text-green-300",
            totalIsZero && !belowCost && "text-muted-foreground",
            belowCost && "text-rose-800 dark:text-rose-300",
          )}>Всього</span
        >
        <CurrencyDisplay
          amount={totalWithVat.toString()}
          suffix="грн."
          class={cn(
            "text-lg font-medium",
            !totalIsZero && "text-green-700 dark:text-green-300",
            totalIsZero && !belowCost && "text-muted-foreground",
            belowCost && "text-rose-800 dark:text-rose-300",
          )}
        />
      </div>
      {#if hasVat && pricesIncludeVat}
        <div class="flex flex-col gap-1 border-l border-border pl-3">
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">Без ПДВ</span>
            <span class="flex items-baseline text-xs text-muted-foreground">
              <span></span>
              <CurrencyDisplay
                amount={vatBase.toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
          <div class="flex items-baseline justify-between">
            <span class="text-xs text-muted-foreground">
              У т.ч. ПДВ ({vatRate}%)
            </span>
            <span class="flex items-baseline text-xs text-muted-foreground">
              <span></span>
              <CurrencyDisplay
                amount={vatAmount.toString()}
                suffix="грн."
                class="text-xs text-muted-foreground"
              />
            </span>
          </div>
        </div>
      {/if}
      {#if belowCost}
        <div
          class="rounded-[8px] border border-rose-500/35 bg-rose-500/10 px-3 py-2.5"
        >
          {#if hasVat}
            <p class="text-xs leading-relaxed text-rose-700 dark:text-rose-400">
              Після сплати ПДВ залишиться {vatBase.toString()} грн., що нижче за собівартість
              робіт і запчастин {totalCost.toString()} грн. (клієнт заплатить {totalWithVat.toString()}
              грн.)
            </p>
          {:else}
            <p class="text-xs leading-relaxed text-rose-700 dark:text-rose-400">
              Кінцева сума зі знижкою становить {totalWithVat.toString()} грн., що
              нижче за собівартість робіт і запчастин {totalCost.toString()} грн.
            </p>
          {/if}
        </div>
      {/if}
    </div>
  </Popover.Content>
</Popover.Root>
