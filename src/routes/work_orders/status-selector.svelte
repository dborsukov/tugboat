<script lang="ts">
  import type { Component } from "svelte";
  import { toast } from "svelte-sonner";

  import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";
  import CanceledIcon from "@lucide/svelte/icons/circle-x";
  import ReceivedIcon from "@lucide/svelte/icons/clipboard-list";
  import AwaitingApprovalIcon from "@lucide/svelte/icons/file-clock";
  import ReadyForPickupIcon from "@lucide/svelte/icons/key";
  import AwaitingPartsIcon from "@lucide/svelte/icons/package";
  import PaidIcon from "@lucide/svelte/icons/receipt";
  import InDiagnosticsIcon from "@lucide/svelte/icons/stethoscope";
  import WorkingIcon from "@lucide/svelte/icons/wrench";

  import {
    commands,
    type WorkOrder,
    type WorkOrderStatus,
  } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { cn } from "$lib/utils";

  import Badge from "$lib/components/ui/badge/badge.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  let {
    work_order,
    status = "Received",
    interactive = true,
  }: {
    work_order?: WorkOrder;
    status?: WorkOrderStatus;
    interactive?: boolean;
  } = $props();

  let loading = $state(false);
  const localStatus = $derived(
    work_order != undefined ? work_order.status : status,
  );

  const statuses: WorkOrderStatus[] = [
    "Received",
    "InDiagnostics",
    "AwaitingApproval",
    "AwaitingParts",
    "Working",
    "ReadyForPickup",
    "Paid",
    "Canceled",
  ];

  const statusStyles: Record<
    WorkOrderStatus,
    {
      text: string;
      icon: Component;
      base: string;
      highlighted: string;
      rounding: string;
      border: string;
    }
  > = {
    Received: {
      text: "Прийнято",
      icon: ReceivedIcon,
      base: "bg-slate-100 text-slate-700",
      highlighted:
        "data-highlighted:bg-slate-200 data-highlighted:text-slate-900",
      rounding: "rounded-b-none",
      border: "border-slate-300",
    },
    InDiagnostics: {
      text: "На діагностиці",
      icon: InDiagnosticsIcon,
      base: "bg-sky-100 text-sky-800",
      highlighted: "data-highlighted:bg-sky-200 data-highlighted:text-sky-900",
      rounding: "",
      border: "border-sky-300",
    },
    AwaitingApproval: {
      text: "Очікує узгодження",
      icon: AwaitingApprovalIcon,
      base: "bg-amber-100 text-amber-700",
      highlighted:
        " data-highlighted:bg-amber-200 data-highlighted:text-amber-900",
      rounding: "",
      border: "border-amber-300",
    },
    AwaitingParts: {
      text: "Очікує запчастини",
      icon: AwaitingPartsIcon,
      base: "bg-violet-100 text-violet-700",
      highlighted:
        " data-highlighted:bg-violet-200 data-highlighted:text-violet-900",
      rounding: "",
      border: "border-violet-300",
    },
    Working: {
      text: "В роботі",
      icon: WorkingIcon,
      base: "bg-blue-100 text-blue-700",
      highlighted:
        " data-highlighted:bg-blue-200 data-highlighted:text-blue-900",
      rounding: "",
      border: "border-blue-300",
    },
    ReadyForPickup: {
      text: "Готово до видачі",
      icon: ReadyForPickupIcon,
      base: "bg-emerald-100 text-emerald-700",
      highlighted:
        " data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900",
      rounding: "",
      border: "border-emerald-300",
    },
    Paid: {
      text: "Сплачено",
      icon: PaidIcon,
      base: "bg-teal-100 text-teal-700",
      highlighted:
        " data-highlighted:bg-teal-200 data-highlighted:text-teal-900",
      rounding: "",
      border: "border-teal-300",
    },
    Canceled: {
      text: "Скасовано",
      icon: CanceledIcon,
      base: "bg-rose-100 text-rose-700",
      highlighted:
        " data-highlighted:bg-rose-200 data-highlighted:text-rose-900",
      rounding: "rounded-t-none",
      border: "border-rose-300",
    },
  };

  const styles = $derived(statusStyles[localStatus]);

  async function updateStatus(newValue: string) {
    if (work_order === undefined) {
      return;
    }
    loading = true;
    const typedStatus = newValue as WorkOrderStatus;
    const payload = {
      ...work_order,
      status: typedStatus,
    };
    console.log(payload);
    let res = await commands.updateWorkOrder(payload);
    if (res.status === "ok") {
      appStore.updateWorkOrder(res.data);
    } else {
      toast.error("Виникла помилка", {
        description: res.error.message,
        position: "bottom-left",
      });
    }
    loading = false;
  }
</script>

{#if interactive}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <Button
          variant="outline"
          class={cn(
            loading && "opacity-75 pointer-events-none",
            styles.base,
            styles.highlighted,
            styles.border,
          )}
          {...props}
        >
          <styles.icon />
          <p>{styles.text}</p>
          {#if loading}
            <Spinner />
          {:else}
            <ChevronsUpDown />
          {/if}
        </Button>
      {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content class="w-55" align="end">
      <DropdownMenu.RadioGroup
        value={localStatus}
        onValueChange={(newValue) => updateStatus(newValue)}
      >
        {#each statuses as s}
          {@const localStyles = statusStyles[s]}
          <DropdownMenu.RadioItem
            value={s}
            class={cn(
              localStyles.base,
              localStyles.highlighted,
              localStyles.rounding,
            )}
          >
            <localStyles.icon />
            <p>{localStyles.text}</p>
          </DropdownMenu.RadioItem>
        {/each}
      </DropdownMenu.RadioGroup>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
{:else}
  <div
    class={cn(
      "flex items-center justify-start gap-2 border rounded max-h-5 px-3 py-5",
      styles.base,
      styles.highlighted,
      styles.border,
    )}
  >
    <styles.icon />
    <p>{styles.text}</p>
  </div>
{/if}
