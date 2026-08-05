<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { goto } from "$app/navigation";

  import { commands } from "$lib/bindings";

  import { unwrap } from "$lib/misc";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import ClientMutationSheet from "../client-mutation-sheet.svelte";
  import { invalidateClientsLists } from "../queries";

  let { clientId }: { clientId: number } = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const createClientDeleteMutation = createMutation(() => ({
    mutationFn: (id: number) => unwrap(commands.deleteClient(id)),
    onSuccess: async () => {
      invalidateClientsLists(queryClient);
      await goto("/clients");
    },
    onError: (error) => {
      toast.error("Виникла помилка", {
        description: error.message,
        position: "bottom-left",
      });
    },
  }));
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

<ClientMutationSheet id={clientId} bind:open={isSheetOpen} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={() => createClientDeleteMutation.mutate(clientId)}
  description="Цю дію неможливо відмінити. Усі пов'язані з цим клієнтом дані будуть видалені."
/>
