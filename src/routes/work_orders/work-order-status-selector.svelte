<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import LockIcon from "@lucide/svelte/icons/lock";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import {
    sheetLabel,
    sheetPrimaryButton,
    sheetSelectTrigger,
  } from "$lib/components/sheet/field-styles";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";

  import {
    type PartItem,
    type PaymentMethod,
    type WorkOrder,
    type WorkOrderStatus,
  } from "$lib/bindings";
  import { prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import { partStatusStylesRecord } from "./[workorderId]/part-status";
  import { paymentMethodLabels } from "./payment-method";
  import {
    createUpdatePartItemMutation,
    createUpdateWorkOrderMutation,
  } from "./queries";
  import {
    getAffectedParts,
    selectableStatuses,
    statusEffects,
    workOrderStatusStylesRecord,
  } from "./work-order-status";

  let {
    workOrder,
    partItems = [],
    isLocked = false,
    class: className,
  }: {
    workOrder?: WorkOrder;
    partItems?: PartItem[];
    isLocked?: boolean;
    class?: string;
  } = $props();

  const localStatus = $derived(workOrder?.status ?? "Received");
  const styles = $derived(workOrderStatusStylesRecord[localStatus]);
  const availableStatuses = $derived(
    selectableStatuses(workOrder?.type ?? "Regular"),
  );

  const queryClient = useQueryClient();
  const updateWorkOrderMutation = createUpdateWorkOrderMutation(queryClient);
  const updatePartItemMutation = createUpdatePartItemMutation(queryClient);

  let dialogOpen = $state(false);

  const isProcessing = $derived(
    updateWorkOrderMutation.isPending || updatePartItemMutation.isPending,
  );

  // svelte-ignore state_referenced_locally
  let selectedStatus = $state<WorkOrderStatus>(localStatus);
  let paymentMethod = $state<PaymentMethod | undefined>(undefined);
  let excludedPartIds = $state<Set<number>>(new Set());
  let paymentTrigger = $state<HTMLElement | null>(null);
  let focusPayment = $state(false);

  $effect(() => {
    if (dialogOpen) {
      selectedStatus = localStatus;
      paymentMethod = undefined;
      excludedPartIds = new Set();
      focusPayment = false;
    }
  });

  const initialStatusStyles = $derived(
    workOrderStatusStylesRecord[workOrder?.status ?? "Received"],
  );
  const selectedStyles = $derived(workOrderStatusStylesRecord[selectedStatus]);
  const hasChanges = $derived(selectedStatus !== localStatus);
  const statusEffect = $derived(statusEffects[selectedStatus]);
  const affectedParts = $derived(getAffectedParts(partItems, statusEffect));
  const includedParts = $derived(
    affectedParts.filter((part) => !excludedPartIds.has(part.id)),
  );
  const requiresPaymentInfo = $derived(
    statusEffect?.requiresPaymentInfo ?? false,
  );
  const paymentMissing = $derived(requiresPaymentInfo && paymentMethod == null);
  const canSave = $derived(
    hasChanges && !isProcessing && !isLocked && !paymentMissing,
  );
  const anyPartsIncluded = $derived(includedParts.length > 0);

  function togglePartExclusion(partId: number, included: boolean) {
    const next = new Set(excludedPartIds);
    if (included) {
      next.delete(partId);
    } else {
      next.add(partId);
    }
    excludedPartIds = next;
  }

  function toggleAllParts() {
    excludedPartIds = anyPartsIncluded
      ? new Set(affectedParts.map((part) => part.id))
      : new Set();
  }

  const fieldLabels: Record<string, string> = {
    startedAt: "Дата початку робіт",
    completedAt: "Дата завершення",
    paidAt: "Дата оплати",
  };

  const fieldPreview = $derived.by(() => {
    if (!workOrder || !statusEffect?.workOrderFields) return [];
    const updates = statusEffect.workOrderFields(workOrder);
    return Object.entries(updates)
      .filter(
        ([key, value]) =>
          value !== undefined && !(workOrder as Record<string, unknown>)[key],
      )
      .map(([key, value]) => ({
        key,
        label: fieldLabels[key] ?? key,
        value: prettifyIsoDateString(value as string, "table-datetime"),
      }));
  });

  async function handleSave() {
    if (!canSave || workOrder === undefined) return;

    const affected = includedParts;

    if (statusEffect?.parts && affected.length > 0) {
      // Sequential on purpose to get reliable .isPending
      // TODO: add dedicated bulk update mutation for parts
      for (const part of affected) {
        await updatePartItemMutation.mutateAsync({
          id: part.id,
          payload: {
            status: statusEffect.parts.to,
            code: part.code,
            name: part.name,
            quantity: part.quantity,
            unitType: part.unitType,
            baseCost: part.baseCost,
            markup: part.markup,
            discountType: part.discountType,
            discountValue: part.discountValue,
            isFree: part.isFree,
            vatRate: part.vatRate,
            vatScheme: part.vatScheme,
            purchaseVatRate: part.purchaseVatRate,
            purchasePriceIncludesVat: part.purchasePriceIncludesVat,
            purchaseScheme: part.purchaseScheme,
            supplierName: part.supplierName,
            supplierCode: part.supplierCode,
            invoiceNumber: part.invoiceNumber,
            invoiceDate: part.invoiceDate,
            vatCredited: part.vatCredited,
          },
        });
      }
    }

    const fieldUpdates = statusEffect?.workOrderFields?.(workOrder) ?? {};

    await updateWorkOrderMutation.mutateAsync({
      ...workOrder,
      ...fieldUpdates,
      status: selectedStatus,
      ...(requiresPaymentInfo ? { paymentMethod } : {}),
    });

    dialogOpen = false;
  }

  function handleOpenChange(next: boolean) {
    if (isProcessing) return;
    dialogOpen = next;
  }

  $effect(() => {
    if (!focusPayment || !requiresPaymentInfo) return;
    focusPayment = false;
    void tick().then(() => paymentTrigger?.focus());
  });

  $effect(() => {
    if (!dialogOpen) return;
    const onKeydown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
        event.preventDefault();
        void handleSave();
      }
    };
    window.addEventListener("keydown", onKeydown, true);
    return () => window.removeEventListener("keydown", onKeydown, true);
  });
</script>

<button
  type="button"
  disabled={workOrder === undefined}
  onclick={() => (dialogOpen = true)}
  class={cn(
    "inline-flex h-10 shrink-0 items-center justify-center gap-2 rounded-4xl border-0 px-3.5 text-base font-medium transition outline-none select-none hover:cursor-pointer hover:brightness-95 focus-visible:ring-3 focus-visible:ring-ring/30 disabled:pointer-events-none disabled:opacity-50",
    styles.base,
    className,
  )}
>
  <styles.icon class="size-5 shrink-0" />
  <span>{styles.text}</span>
  <ChevronRightIcon class="size-4 opacity-80" />
</button>

{#if workOrder}
  <Dialog.Root open={dialogOpen} onOpenChange={handleOpenChange}>
    <Dialog.Content
      class="flex max-h-[calc(100dvh-var(--overlay-inset,0px)-2rem)] w-full flex-col gap-0 overflow-hidden rounded-[16px] p-0 sm:max-w-[520px]"
    >
      <Dialog.Header
        class="shrink-0 border-b border-border px-6 pt-5 pb-4 pr-14"
      >
        <Dialog.Title
          class="text-[17px] leading-tight font-semibold tracking-[-0.01em]"
        >
          Змінити статус замовлення
        </Dialog.Title>
        <Dialog.Description class="text-[13px] leading-snug">
          Нижче будуть вказані усі заплановані зміни.
        </Dialog.Description>
      </Dialog.Header>

      <div class="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        <div class="flex flex-col gap-6">
          {#if isLocked}
            <div
              class="flex items-center gap-2 rounded-[8px] border border-border px-3 py-2 text-sm text-muted-foreground"
            >
              <LockIcon class="size-4 shrink-0" />
              <span>Замовлення заблоковано від змін</span>
            </div>
          {/if}

          {#if isLocked}
            <div class="space-y-1.5">
              <Label class={sheetLabel}>Зараз</Label>
              <div
                class="flex h-9.5 w-fit items-center gap-2 rounded-[8px] border border-border bg-muted/40 px-3 text-sm font-medium whitespace-nowrap"
              >
                <initialStatusStyles.icon
                  class="size-4 shrink-0 text-muted-foreground"
                />
                {initialStatusStyles.text}
              </div>
            </div>
          {:else}
            <div
              class="grid grid-cols-[auto_auto_minmax(0,1fr)] items-end gap-2.5"
            >
              <div class="min-w-0 space-y-1.5">
                <Label class={sheetLabel}>Зараз</Label>
                <div
                  class="flex h-9.5 items-center gap-2 rounded-[8px] border border-border bg-muted/40 px-3 text-sm font-medium whitespace-nowrap"
                >
                  <initialStatusStyles.icon
                    class="size-4 shrink-0 text-muted-foreground"
                  />
                  {initialStatusStyles.text}
                </div>
              </div>

              <span
                class="pb-2.5 text-base leading-none text-muted-foreground"
                aria-hidden="true">→</span
              >

              <div class="min-w-0 space-y-1.5">
                <Label class={sheetLabel}>Новий статус</Label>
                <Select.Root
                  type="single"
                  value={selectedStatus}
                  onValueChange={(v) => {
                    if (!v) return;
                    const next = v as WorkOrderStatus;
                    selectedStatus = next;
                    if (statusEffects[next]?.requiresPaymentInfo) {
                      focusPayment = true;
                    }
                  }}
                >
                  <Select.Trigger
                    class={cn(
                      selectedStyles.base,
                      selectedStyles.border,
                      "h-9.5 w-full justify-between rounded-[8px] border px-3 text-sm font-medium data-[size=default]:h-9.5 data-[size=default]:rounded-[8px]",
                    )}
                  >
                    <span class="flex min-w-0 items-center gap-2">
                      <selectedStyles.icon class="size-4 shrink-0" />
                      <span class="truncate">{selectedStyles.text}</span>
                    </span>
                  </Select.Trigger>
                  <Select.Content>
                    {#each availableStatuses as s (s)}
                      {@const optionStyles = workOrderStatusStylesRecord[s]}
                      <Select.Item
                        value={s}
                        label={optionStyles.text}
                        class={cn(
                          optionStyles.base,
                          optionStyles.highlighted,
                          "gap-2 rounded-none",
                          optionStyles.rounding,
                        )}
                      >
                        <optionStyles.icon class="size-4" />
                        {optionStyles.text}
                      </Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
              </div>
            </div>
          {/if}

          {#if !isLocked && requiresPaymentInfo}
            <div class="space-y-1.5">
              <Label class={sheetLabel}>
                Спосіб оплати<RequiredMark />
              </Label>
              <Select.Root
                type="single"
                value={paymentMethod}
                onValueChange={(v) => {
                  if (!v) return;
                  paymentMethod = v as PaymentMethod;
                }}
              >
                <Select.Trigger
                  bind:ref={paymentTrigger}
                  class={cn(
                    sheetSelectTrigger,
                    "w-full",
                    !paymentMethod && "text-muted-foreground",
                  )}
                >
                  {paymentMethod
                    ? paymentMethodLabels[paymentMethod]
                    : "Оберіть спосіб оплати"}
                </Select.Trigger>
                <Select.Content>
                  {#each Object.entries(paymentMethodLabels) as [value, label] (value)}
                    <Select.Item {value}>{label}</Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </div>
          {/if}

          {#if !hasChanges}
            <p class="text-[13px] text-muted-foreground">Це поточний статус.</p>
          {:else}
            {#if fieldPreview.length > 0}
              <section class="space-y-3">
                <div class="flex items-center gap-2">
                  <h3
                    class="shrink-0 text-[11.5px] font-bold tracking-[0.06em] whitespace-nowrap text-muted-foreground/80 uppercase"
                  >
                    Зміни в замовленні
                  </h3>
                  <span class="h-px min-w-4 flex-1 bg-border" aria-hidden="true"
                  ></span>
                  <span class="shrink-0 text-[11px] text-muted-foreground/70">
                    автоматично
                  </span>
                </div>
                <div
                  class="overflow-hidden rounded-[8px] border border-border bg-card/40"
                >
                  <div class="divide-y divide-border">
                    {#each fieldPreview as field (field.key)}
                      <div
                        class="flex h-10 items-center justify-between gap-3 px-3"
                      >
                        <span class="text-[12.5px]">{field.label}</span>
                        <span class="text-xs text-muted-foreground">
                          {field.value}
                        </span>
                      </div>
                    {/each}
                  </div>
                </div>
              </section>
            {/if}

            {#if affectedParts.length > 0 && statusEffect?.parts}
              {@const targetStyles =
                partStatusStylesRecord[statusEffect.parts.to]}
              <section class="space-y-3">
                <div class="flex items-center gap-2">
                  <h3
                    class="shrink-0 text-[11.5px] font-bold tracking-[0.06em] whitespace-nowrap text-muted-foreground/80 uppercase"
                  >
                    Запчастини · {includedParts.length} з {affectedParts.length}
                  </h3>
                  <span class="h-px min-w-4 flex-1 bg-border" aria-hidden="true"
                  ></span>
                  {#if affectedParts.length > 1}
                    <button
                      type="button"
                      class="shrink-0 text-[11px] font-medium text-brand transition-colors hover:text-brand-hover"
                      onclick={toggleAllParts}
                    >
                      {anyPartsIncluded ? "Зняти всі" : "Обрати всі"}
                    </button>
                  {/if}
                </div>
                <div
                  class="overflow-hidden rounded-[8px] border border-border bg-card/40"
                >
                  <div
                    class="max-h-[240px] divide-y divide-border overflow-y-auto"
                  >
                    {#each affectedParts as part (part.id)}
                      {@const fromStyles = partStatusStylesRecord[part.status]}
                      {@const partIncluded = !excludedPartIds.has(part.id)}
                      <div
                        class={cn(
                          "flex h-[46px] items-center gap-2.5 px-3",
                          !partIncluded && "opacity-50",
                        )}
                      >
                        <Checkbox
                          checked={partIncluded}
                          class="size-4 rounded-[4px] data-checked:border-brand data-checked:bg-brand data-checked:text-white dark:data-checked:border-brand dark:data-checked:bg-brand dark:data-checked:text-white"
                          onCheckedChange={(checked: boolean) =>
                            togglePartExclusion(part.id, checked === true)}
                          aria-label={`Не змінювати статус для ${part.name}`}
                        />
                        <span class="min-w-0 flex-1 truncate text-[13px]">
                          {part.name}
                        </span>
                        <span
                          class="flex shrink-0 items-center gap-1.5 text-[11px]"
                        >
                          <span
                            class="rounded-[5px] bg-muted/60 px-1.5 py-0.5 text-muted-foreground"
                          >
                            {fromStyles.text}
                          </span>
                          <span
                            class="text-muted-foreground/60"
                            aria-hidden="true">→</span
                          >
                          <span
                            class={cn(
                              "rounded-[5px] px-1.5 py-0.5 font-medium",
                              targetStyles.base,
                            )}
                          >
                            {targetStyles.text}
                          </span>
                        </span>
                      </div>
                    {/each}
                  </div>
                </div>
              </section>
            {/if}

            {#if fieldPreview.length === 0 && affectedParts.length === 0 && !requiresPaymentInfo}
              <div class="text-[13px] text-muted-foreground">
                <p>Статус буде змінено на «{selectedStyles.text}».</p>
                <p>Інших змін не буде.</p>
              </div>
            {/if}
          {/if}
        </div>
      </div>

      <div
        class="flex shrink-0 items-center gap-2 border-t border-border px-6 py-4"
      >
        {#if paymentMissing}
          <p class="mr-auto text-xs text-muted-foreground">
            Оберіть спосіб оплати
          </p>
        {:else}
          <span class="mr-auto"></span>
        {/if}
        <Button
          variant="ghost"
          class="h-9.5 shrink-0 rounded-[8px] px-2.5 text-xs font-medium text-muted-foreground hover:text-foreground"
          onclick={() => handleOpenChange(false)}
          disabled={isProcessing}
        >
          Скасувати
        </Button>
        <Button
          variant="brand"
          class={sheetPrimaryButton}
          onclick={handleSave}
          disabled={!canSave}
          aria-busy={isProcessing}
        >
          {#if isProcessing}
            <Spinner class="size-3.5" />
          {/if}
          Зберегти
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Root>
{/if}
