<script lang="ts">
  import { toast } from "svelte-sonner";

  import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";

  import {
    commands,
    type WorkOrder,
    type WorkOrderStatus,
  } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { cn } from "$lib/utils";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { statuses, statusStyles } from "./work-order-status";

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
