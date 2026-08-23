<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import { createDeleteWorkOrderMutation } from "../queries";
  import WorkOrderMutationSheet from "../work-order-mutation-sheet.svelte";

  let {
    workOrderId,
    isLocked = false,
  }: { workOrderId: string; isLocked?: boolean } = $props();
  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const deleteWorkOrderMutation = createDeleteWorkOrderMutation(queryClient);

  async function handleDelete() {
    deleteWorkOrderMutation.mutate(workOrderId, {
      onSuccess: async () => {
        isDeleteDialogOpen = false;
        await goto("/work_orders");
      },
    });
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon-lg" })}
  >
    <EllipsisIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-40" align="end">
    <DropdownMenu.Group>
      <DropdownMenu.Item onSelect={() => (isSheetOpen = true)}>
        Редагувати
        <EditIcon class="ml-auto" />
      </DropdownMenu.Item>
      <DropdownMenu.Item
        // FIXME: figure out how to make icon red on focus(hover)
        // class="text-destructive focus:text-destructive"
        onSelect={() => (isDeleteDialogOpen = true)}
      >
        Видалити
        <TrashIcon class="ml-auto" />
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<WorkOrderMutationSheet
  bind:open={isSheetOpen}
  {isLocked}
  mode="edit"
  {workOrderId}
/>
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити. Усі пов'язані з цим замовленням дані будуть видалені."
/>
