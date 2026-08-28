<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import { goto } from "$app/navigation";

  import CopyIcon from "@lucide/svelte/icons/copy";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import { copyText } from "$lib/hooks/use-clipboard.svelte";

  import ClientMutationSheet from "../client-mutation-sheet.svelte";
  import { createDeleteClientMutation } from "../queries";

  let {
    clientId,
    phone,
    email,
  }: {
    clientId: number;
    phone: string | null;
    email: string | null;
  } = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const deleteClientMutation = createDeleteClientMutation(queryClient);

  function handleDelete() {
    deleteClientMutation.mutate(clientId, {
      onSuccess: async () => {
        await goto("/clients");
      },
    });
  }

  async function copyValue(value: string, label: string) {
    const status = await copyText(value);
    if (status === "success") {
      toast.success(`${label} скопійовано`, { position: "bottom-left" });
    } else {
      toast.error("Не вдалося скопіювати", {
        description: label,
        position: "bottom-right",
      });
    }
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon" })}
  >
    <EllipsisIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-48" align="end">
    <DropdownMenu.Item onSelect={() => (isSheetOpen = true)}>
      <EditIcon />
      Редагувати
    </DropdownMenu.Item>

    <DropdownMenu.Separator />

    <DropdownMenu.Label>Телефон</DropdownMenu.Label>
    <DropdownMenu.Item
      disabled={!phone}
      onSelect={() => phone && copyValue(phone, "Телефон")}
    >
      <CopyIcon />
      Копіювати
    </DropdownMenu.Item>

    <DropdownMenu.Separator />

    <DropdownMenu.Label>Email</DropdownMenu.Label>
    <DropdownMenu.Item
      disabled={!email}
      onSelect={() => email && copyValue(email, "Email")}
    >
      <CopyIcon />
      Копіювати
    </DropdownMenu.Item>

    <DropdownMenu.Separator />

    <DropdownMenu.Item
      variant="destructive"
      onSelect={() => (isDeleteDialogOpen = true)}
    >
      <TrashIcon />
      Видалити
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<ClientMutationSheet {clientId} bind:open={isSheetOpen} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити. Усі пов'язані з цим клієнтом дані будуть видалені."
/>
