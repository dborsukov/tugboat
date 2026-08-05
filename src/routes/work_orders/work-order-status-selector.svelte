<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { type WorkOrder, type WorkOrderStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { createUpdateWorkOrderMutation } from "./queries";
  import { statuses, statusStyles } from "./work-order-status";

  let {
    workOrder,
    status = "Received",
    interactive = true,
  }: {
    workOrder?: WorkOrder;
    status?: WorkOrderStatus;
    interactive?: boolean;
  } = $props();

  const localStatus = $derived(
    workOrder != undefined ? workOrder.status : status,
  );
  const styles = $derived(statusStyles[localStatus]);

  const queryClient = useQueryClient();
  const updateWorkOrderMutation = createUpdateWorkOrderMutation(queryClient);

  function updateStatus(newValue: string) {
    if (workOrder === undefined) return;
    updateWorkOrderMutation.mutate({
      ...workOrder,
      status: newValue as WorkOrderStatus,
    });
  }
</script>

{#if interactive}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <Button
          variant="outline"
          class={cn(
            updateWorkOrderMutation.isPending &&
              "opacity-75 pointer-events-none",
            styles.base,
            styles.highlighted,
            styles.border,
          )}
          {...props}
        >
          <styles.icon />
          <p>{styles.text}</p>
          {#if updateWorkOrderMutation.isPending}
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
