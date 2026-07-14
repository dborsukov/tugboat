<script lang="ts">
  import { toast } from "svelte-sonner";

  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { commands } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import WorkOrderPartItemMutationSheet from "./work-order-parts-item-mutation-sheet.svelte";

  let { id }: { id: number } = $props();
  let isSheetOpen: boolean = $state(false);
  let isDeleteDialogOpen: boolean = $state(false);

  async function handleDelete() {
    let res = await commands.deleteWorkOrderPartItem(id);
    if (res.status === "ok") {
      appStore.removeWorkOrderPartItem(id);
    } else {
      toast.error("Виникла помилка", {
        description: res.error.message,
        position: "bottom-center",
      });
    }
    isDeleteDialogOpen = false;
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon">
        <EllipsisIcon />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-40" align="end">
    <DropdownMenu.Group>
      <DropdownMenu.Label>Дії</DropdownMenu.Label>
      <DropdownMenu.Item
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        Редагувати
        <EditIcon class="ml-auto" />
      </DropdownMenu.Item>
      <DropdownMenu.Item
        onclick={() => {
          isDeleteDialogOpen = !isDeleteDialogOpen;
        }}
      >
        Видалити
        <TrashIcon class="ml-auto" />
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<WorkOrderPartItemMutationSheet bind:open={isSheetOpen} {id} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити."
/>
