<script lang="ts">
  import type { Component } from "svelte";

  import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";
  import InstalledIcon from "@lucide/svelte/icons/circle-check";
  import AwaitingReturnIcon from "@lucide/svelte/icons/history";
  import InStockIcon from "@lucide/svelte/icons/package-2";
  import ReturnedIcon from "@lucide/svelte/icons/rotate-ccw";
  import QuotedIcon from "@lucide/svelte/icons/scan-search";
  import WarrantyIcon from "@lucide/svelte/icons/shield";
  import OrderedIcon from "@lucide/svelte/icons/truck";
  import InstallingIcon from "@lucide/svelte/icons/wrench";

  import { type PartStatus } from "$lib/bindings";

  import { cn } from "$lib/utils";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  let {
    status = $bindable(),
    interactive = true,
  }: {
    status: PartStatus;
    interactive?: boolean;
  } = $props();

  let loading = $state(false);

  const statuses: PartStatus[] = [
    "Quoted",
    "Ordered",
    "InStock",
    "Installing",
    "Installed",
    "AwaitingReturn",
    "Returned",
    "Warranty",
  ];

  const statusStyles: Record<
    PartStatus,
    {
      text: string;
      icon: Component;
      base: string;
      highlighted: string;
      rounding: string;
      border: string;
    }
  > = {
    Quoted: {
      text: "Підбір",
      icon: QuotedIcon,
      base: "bg-slate-100 text-slate-700",
      highlighted:
        "data-highlighted:bg-slate-200 data-highlighted:text-slate-900",
      rounding: "rounded-b-none",
      border: "border-slate-300",
    },
    Ordered: {
      text: "Замовлено",
      icon: OrderedIcon,
      base: "bg-sky-100 text-sky-800",
      highlighted: "data-highlighted:bg-sky-200 data-highlighted:text-sky-900",
      rounding: "",
      border: "border-sky-300",
    },
    InStock: {
      text: "В наявності",
      icon: InStockIcon,
      base: "bg-violet-100 text-violet-700",
      highlighted:
        " data-highlighted:bg-violet-200 data-highlighted:text-violet-900",
      rounding: "",
      border: "border-violet-300",
    },
    Installing: {
      text: "Встановлення",
      icon: InstallingIcon,
      base: "bg-blue-100 text-blue-700",
      highlighted:
        " data-highlighted:bg-blue-200 data-highlighted:text-blue-900",
      rounding: "",
      border: "border-blue-300",
    },
    Installed: {
      text: "Встановлено",
      icon: InstalledIcon,
      base: "bg-emerald-100 text-emerald-700",
      highlighted:
        " data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900",
      rounding: "",
      border: "border-emerald-300",
    },
    AwaitingReturn: {
      text: "Очікує на повернення",
      icon: AwaitingReturnIcon,
      base: "bg-amber-100 text-amber-700",
      highlighted:
        " data-highlighted:bg-amber-200 data-highlighted:text-amber-900",
      rounding: "",
      border: "border-amber-300",
    },
    Returned: {
      text: "Повернено",
      icon: ReturnedIcon,
      base: "bg-gray-100 text-gray-700",
      highlighted:
        " data-highlighted:bg-gray-200 data-highlighted:text-gray-900",
      rounding: "",
      border: "border-gray-300",
    },
    Warranty: {
      text: "Гарантійний випадок",
      icon: WarrantyIcon,
      base: "bg-rose-100 text-rose-700",
      highlighted:
        " data-highlighted:bg-rose-200 data-highlighted:text-rose-900",
      rounding: "rounded-t-none",
      border: "border-rose-300",
    },
  };

  const styles = $derived(statusStyles[status]);
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
    <DropdownMenu.Content class="w-65" align="end">
      <DropdownMenu.RadioGroup bind:value={status}>
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
