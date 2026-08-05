<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { commands } from "$lib/bindings";

  import { unwrap } from "$lib/misc";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import { invalidateServiceLists } from "./queries";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let { id }: { id: number } = $props();
  let isSheetOpen: boolean = $state(false);
  let isDeleteDialogOpen: boolean = $state(false);

  const queryClient = useQueryClient();

  const deleteServiceMutation = createMutation(() => ({
    mutationFn: (id: number) => unwrap(commands.deleteService(id)),
    onSuccess: () => {
      invalidateServiceLists(queryClient);
      isDeleteDialogOpen = false;
    },
    onError: (error) => {
      toast.error("Виникла помилка", {
        description: error.message,
        position: "bottom-left",
      });
    },
  }));

  async function handleDelete() {
    deleteServiceMutation.mutate(id);
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

<ServiceMutationSheet bind:open={isSheetOpen} {id} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити."
/>
