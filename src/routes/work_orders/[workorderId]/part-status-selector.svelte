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

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { type PartStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

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
      base: "bg-slate-100 text-slate-700 dark:bg-slate-500/15 dark:text-slate-300",
      highlighted:
        "data-highlighted:bg-slate-200 data-highlighted:text-slate-900 dark:data-highlighted:bg-slate-500/25 dark:data-highlighted:text-slate-100",
      rounding: "rounded-t-xl",
      border: "border-slate-300 dark:border-slate-500/30",
    },
    Ordered: {
      text: "Замовлено",
      icon: OrderedIcon,
      base: "bg-sky-100 text-sky-800 dark:bg-sky-500/15 dark:text-sky-300",
      highlighted:
        "data-highlighted:bg-sky-200 data-highlighted:text-sky-900 dark:data-highlighted:bg-sky-500/25 dark:data-highlighted:text-sky-100",
      rounding: "",
      border: "border-sky-300 dark:border-sky-500/30",
    },
    InStock: {
      text: "В наявності",
      icon: InStockIcon,
      base: "bg-violet-100 text-violet-700 dark:bg-violet-500/15 dark:text-violet-300",
      highlighted:
        "data-highlighted:bg-violet-200 data-highlighted:text-violet-900 dark:data-highlighted:bg-violet-500/25 dark:data-highlighted:text-violet-100",
      rounding: "",
      border: "border-violet-300 dark:border-violet-500/30",
    },
    Installing: {
      text: "Встановлення",
      icon: InstallingIcon,
      base: "bg-blue-100 text-blue-700 dark:bg-blue-500/15 dark:text-blue-300",
      highlighted:
        "data-highlighted:bg-blue-200 data-highlighted:text-blue-900 dark:data-highlighted:bg-blue-500/25 dark:data-highlighted:text-blue-100",
      rounding: "",
      border: "border-blue-300 dark:border-blue-500/30",
    },
    Installed: {
      text: "Встановлено",
      icon: InstalledIcon,
      base: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
      highlighted:
        "data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900 dark:data-highlighted:bg-emerald-500/25 dark:data-highlighted:text-emerald-100",
      rounding: "",
      border: "border-emerald-300 dark:border-emerald-500/30",
    },
    AwaitingReturn: {
      text: "Очікує на повернення",
      icon: AwaitingReturnIcon,
      base: "bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300",
      highlighted:
        "data-highlighted:bg-amber-200 data-highlighted:text-amber-900 dark:data-highlighted:bg-amber-500/25 dark:data-highlighted:text-amber-100",
      rounding: "",
      border: "border-amber-300 dark:border-amber-500/30",
    },
    Returned: {
      text: "Повернено",
      icon: ReturnedIcon,
      base: "bg-gray-100 text-gray-700 dark:bg-gray-500/15 dark:text-gray-300",
      highlighted:
        "data-highlighted:bg-gray-200 data-highlighted:text-gray-900 dark:data-highlighted:bg-gray-500/25 dark:data-highlighted:text-gray-100",
      rounding: "",
      border: "border-gray-300 dark:border-gray-500/30",
    },
    Warranty: {
      text: "Гарантійний випадок",
      icon: WarrantyIcon,
      base: "bg-rose-100 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300",
      highlighted:
        "data-highlighted:bg-rose-200 data-highlighted:text-rose-900 dark:data-highlighted:bg-rose-500/25 dark:data-highlighted:text-rose-100",
      rounding: "rounded-b-xl",
      border: "border-rose-300 dark:border-rose-500/30",
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
              "rounded-none",
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
