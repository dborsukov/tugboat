<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { type WorkOrder, type WorkOrderStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { createUpdateWorkOrderMutation } from "./queries";
  import { statuses, workOrderStatusStylesRecord } from "./work-order-status";

  let {
    workOrder,
    status = "Received",
    class: className,
  }: {
    workOrder?: WorkOrder;
    status?: WorkOrderStatus;
    class?: string;
  } = $props();

  const localStatus = $derived(
    workOrder != undefined ? workOrder.status : status,
  );
  const styles = $derived(workOrderStatusStylesRecord[localStatus]);

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

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button
        size="lg"
        variant="outline"
        class={cn(
          updateWorkOrderMutation.isPending && "opacity-75 pointer-events-none",
          styles.base,
          styles.highlighted,
          "text-base px-2 border-none",
          className,
        )}
        {...props}
      >
        <styles.icon class="size-5" />
        <p>{styles.text}</p>
        {#if updateWorkOrderMutation.isPending}
          <Spinner />
        {:else}
          <ChevronDownIcon class="size-5" />
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
        {@const localStyles = workOrderStatusStylesRecord[s]}
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
