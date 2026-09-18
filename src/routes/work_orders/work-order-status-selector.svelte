<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import LockIcon from "@lucide/svelte/icons/lock";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as Item from "$lib/components/ui/item/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

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
    status = "Received",
    isLocked = false,
    class: className,
  }: {
    workOrder?: WorkOrder;
    partItems?: PartItem[];
    status?: WorkOrderStatus;
    isLocked?: boolean;
    class?: string;
  } = $props();

  const localStatus = $derived(
    workOrder != undefined ? workOrder.status : status,
  );
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
  let paymentMethodMissing = $state(false);
  let excludedPartIds = $state<Set<number>>(new Set());

  $effect(() => {
    if (dialogOpen) {
      selectedStatus = localStatus;
      paymentMethod = undefined;
      paymentMethodMissing = false;
      excludedPartIds = new Set();
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
  const canSave = $derived(hasChanges && !isProcessing && !isLocked);

  function togglePartExclusion(partId: number, included: boolean) {
    const next = new Set(excludedPartIds);
    if (included) {
      next.delete(partId);
    } else {
      next.add(partId);
    }
    excludedPartIds = next;
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
        value: prettifyIsoDateString(value as string, "datetime"),
      }));
  });

  async function handleSave() {
    if (!canSave || workOrder === undefined) return;

    if (requiresPaymentInfo && !paymentMethod) {
      paymentMethodMissing = true;
      return;
    }

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
          },
        });
      }
    }

    const fieldUpdates = statusEffect?.workOrderFields?.(workOrder) ?? {};

    console.log("workOrder: ", workOrder);

    console.log("canSave: ", canSave);

    console.log("requiresPaymentInfo: ", requiresPaymentInfo);

    console.log("payload: ", {
      ...workOrder,
      ...fieldUpdates,
      status: selectedStatus,
      ...(requiresPaymentInfo ? { paymentMethod } : {}),
    });

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
</script>

<Button
  size="lg"
  variant="outline"
  disabled={workOrder === undefined}
  onclick={() => (dialogOpen = true)}
  class={cn(
    styles.base,
    styles.highlighted,
    "text-base px-2 border-none",
    className,
  )}
>
  <styles.icon class="size-5" />
  <p>{styles.text}</p>
  <ChevronRightIcon class="size-5 opacity-75" />
</Button>

{#if workOrder}
  <Dialog.Root open={dialogOpen} onOpenChange={handleOpenChange}>
    <Dialog.Content class="gap-4 select-none!">
      <Dialog.Header>
        <Dialog.Title>Змінити статус замовлення</Dialog.Title>
        <Dialog.Description
          >Нижче будуть вказані усі заплановані зміни.</Dialog.Description
        >
      </Dialog.Header>

      {#if isLocked}
        <div
          class="flex items-center gap-2 rounded border border-border px-3 py-2 text-sm text-muted-foreground"
        >
          <LockIcon class="size-4 shrink-0" />
          <span>Замовлення заблоковано від змін</span>
        </div>
      {/if}

      <Item.Root class="p-0">
        <Item.Content>
          <Item.Title>Поточний статус</Item.Title>
        </Item.Content>
        <Item.Actions>
          <div
            class={cn(
              initialStatusStyles.base,
              "w-fit px-3 py-1 rounded text-base select-none",
            )}
          >
            <span class="flex items-center gap-2">
              <initialStatusStyles.icon class="size-5" />
              {initialStatusStyles.text}
            </span>
          </div>
        </Item.Actions>
      </Item.Root>

      {#if !isLocked}
        <Item.Root class="p-0">
          <Item.Content>
            <Item.Title>Новий статус</Item.Title>
          </Item.Content>
          <Item.Actions>
            <Select.Root
              type="single"
              value={selectedStatus}
              onValueChange={(v) =>
                v && (selectedStatus = v as WorkOrderStatus)}
            >
              <Select.Trigger
                class={cn(
                  selectedStyles.base,
                  "p-2 text-base hover:cursor-pointer",
                )}
              >
                <span class="flex items-center gap-2">
                  <selectedStyles.icon class="size-5" />
                  {selectedStyles.text}
                </span>
              </Select.Trigger>
              <Select.Content>
                {#each availableStatuses as s}
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
          </Item.Actions>
        </Item.Root>

        {#if requiresPaymentInfo}
          <Item.Root class="p-0">
            <Item.Content>
              <Item.Title>Метод оплати</Item.Title>
            </Item.Content>
            <Item.Actions>
              <div class="flex flex-col items-end gap-1">
                <Select.Root
                  type="single"
                  value={paymentMethod}
                  onValueChange={(v) => {
                    paymentMethod = v as PaymentMethod;
                    paymentMethodMissing = false;
                  }}
                >
                  <Select.Trigger
                    class={cn(!paymentMethod && "text-muted-foreground")}
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
                {#if paymentMethodMissing}
                  <p class="text-xs text-destructive">Оберіть спосіб оплати</p>
                {/if}
              </div>
            </Item.Actions>
          </Item.Root>
        {/if}

        {#if !hasChanges}
          <p class="text-sm text-muted-foreground">Це поточний статус.</p>
        {:else}
          <div class="flex flex-col gap-4">
            {#if fieldPreview.length > 0}
              <div class="flex flex-col gap-2">
                <h4 class="text-sm font-semibold">Зміни в замовленні</h4>
                <div
                  class="rounded border border-border divide-y divide-border"
                >
                  {#each fieldPreview as field (field.key)}
                    <Item.Root class="px-3 py-2">
                      <Item.Content>
                        <Item.Title class="text-sm font-normal">
                          {field.label}
                        </Item.Title>
                      </Item.Content>
                      <Item.Actions>
                        <span class="text-xs text-muted-foreground">
                          {field.value}
                        </span>
                      </Item.Actions>
                    </Item.Root>
                  {/each}
                </div>
              </div>
            {/if}

            {#if affectedParts.length > 0 && statusEffect?.parts}
              {@const targetStyles =
                partStatusStylesRecord[statusEffect.parts.to]}
              <div class="flex flex-col gap-2">
                <h4 class="text-sm font-semibold">
                  Зміни в запчастинах ({includedParts.length}/{affectedParts.length})
                </h4>
                <div
                  class="max-h-40 overflow-y-auto rounded border border-border divide-y divide-border"
                >
                  {#each affectedParts as part (part.id)}
                    {@const fromStyles = partStatusStylesRecord[part.status]}
                    {@const partIncluded = !excludedPartIds.has(part.id)}
                    <Item.Root
                      class={cn("px-3 py-2", !partIncluded && "opacity-50")}
                    >
                      <Item.Content class="flex-row items-center gap-2">
                        <Checkbox
                          checked={partIncluded}
                          onCheckedChange={(checked: boolean) =>
                            togglePartExclusion(part.id, checked === true)}
                          aria-label={`Не змінювати статус для ${part.name}`}
                        />
                        <Item.Title class="text-sm font-normal truncate">
                          {part.name}
                        </Item.Title>
                      </Item.Content>
                      <Item.Actions>
                        <span class="flex shrink-0 items-center gap-1.5">
                          <span
                            class={cn(
                              "rounded px-1.5 py-0.5 text-xs",
                              fromStyles.base,
                            )}
                          >
                            {fromStyles.text}
                          </span>
                          <ChevronRightIcon
                            class="size-3.5 shrink-0 text-muted-foreground"
                          />
                          <span
                            class={cn(
                              "rounded px-1.5 py-0.5 text-xs",
                              targetStyles.base,
                            )}
                          >
                            {targetStyles.text}
                          </span>
                        </span>
                      </Item.Actions>
                    </Item.Root>
                  {/each}
                </div>
              </div>
            {/if}

            {#if fieldPreview.length === 0 && affectedParts.length === 0 && !requiresPaymentInfo}
              <div class="text-sm text-muted-foreground">
                <p>Статус буде змінено на «{selectedStyles.text}».</p>
                <p>Інших змін не буде.</p>
              </div>
            {/if}
          </div>
        {/if}
      {/if}

      <Dialog.Footer class="mt-2">
        <Button
          variant="outline"
          onclick={() => handleOpenChange(false)}
          disabled={isProcessing}
        >
          Скасувати
        </Button>
        <Button onclick={handleSave} disabled={!canSave}>
          {#if isProcessing}
            <Spinner class="mr-2" />
          {/if}
          Зберегти
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  </Dialog.Root>
{/if}
