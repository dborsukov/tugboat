<script lang="ts">
  import { toast } from "svelte-sonner";

  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { goto } from "$app/navigation";

  import { commands, type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import WorkOrderMutationSheet from "../work-order-mutation-sheet.svelte";

  interface Props {
    work_order: WorkOrder;
  }
  let { work_order }: Props = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  async function handleDelete() {
    let res = await commands.archiveWorkOrder(work_order.id);
    if (res.status === "ok") {
      appStore.archiveWorkOrder(work_order.id);
      await goto("/work_orders");
      toast.success("Замовлення видалено");
    } else {
      toast.error("Виникла помилка", {
        description: res.error.message,
        position: "bottom-center",
      });
    }
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "outline", size: "icon" })}
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

<WorkOrderMutationSheet bind:open={isSheetOpen} {work_order} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити. Усі пов'язані з цим замовленням дані будуть видалені."
/>
