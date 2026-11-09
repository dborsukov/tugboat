<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import type { ClientDeleteImpact } from "$lib/bindings";
  import { copyText } from "$lib/hooks/use-clipboard.svelte";
  import { pluralizeUk } from "$lib/tools";

  import { invalidateCars } from "../../cars/queries";
  import { invalidateWorkOrderLists } from "../../work_orders/queries";
  import ClientMutationSheet from "../client-mutation-sheet.svelte";
  import {
    createArchiveClientMutation,
    createClientDeleteImpactQuery,
    createDeleteClientMutation,
    createRestoreClientMutation,
  } from "../queries";

  let {
    clientId,
    isArchived = false,
    phone,
    email,
  }: {
    clientId: number;
    isArchived?: boolean;
    phone: string | null;
    email: string | null;
  } = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);
  let isArchiveDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const deleteClientMutation = createDeleteClientMutation(queryClient);
  const archiveClientMutation = createArchiveClientMutation(queryClient);
  const restoreClientMutation = createRestoreClientMutation(queryClient);

  const deleteImpactQuery = createClientDeleteImpactQuery(
    () => clientId,
    () => isDeleteDialogOpen,
  );

  const deleteBlocked = $derived(
    (deleteImpactQuery.data?.workOrderCount ?? 0) > 0,
  );

  function describeDeleteImpact(impact: ClientDeleteImpact | undefined) {
    if (!impact) {
      return "Цю дію неможливо відмінити. Клієнта буде видалено безповоротно разом з усіма пов'язаними даними.";
    }
    const parts: string[] = [];
    if (impact.carCount > 0) {
      parts.push(
        `${impact.carCount} ${pluralizeUk(impact.carCount, ["автомобіль", "автомобілі", "автомобілів"])}`,
      );
    }
    if (impact.workOrderCount > 0) {
      parts.push(
        `${impact.workOrderCount} ${pluralizeUk(impact.workOrderCount, ["замовлення", "замовлення", "замовлень"])}`,
      );
    }
    if (impact.laborItemCount > 0) {
      parts.push(
        `${impact.laborItemCount} ${pluralizeUk(impact.laborItemCount, ["робота", "роботи", "робіт"])}`,
      );
    }
    if (impact.partItemCount > 0) {
      parts.push(
        `${impact.partItemCount} ${pluralizeUk(impact.partItemCount, ["запчастина", "запчастини", "запчастин"])}`,
      );
    }
    if (parts.length === 0) {
      return "Цю дію неможливо відмінити. Клієнта буде видалено безповоротно.";
    }
    return `Буде видалено: ${parts.join(", ")}. Цю дію неможливо відмінити.`;
  }

  function invalidateCascades() {
    invalidateCars(queryClient);
    invalidateWorkOrderLists(queryClient);
  }

  function handleDelete() {
    deleteClientMutation.mutate(clientId, {
      onSuccess: async () => {
        await goto("/clients");
      },
    });
  }

  function handleArchive() {
    archiveClientMutation.mutate(clientId, {
      onSuccess: invalidateCascades,
    });
  }

  function handleRestore() {
    restoreClientMutation.mutate(clientId, {
      onSuccess: invalidateCascades,
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
  <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
    {#if isArchived}
      <DropdownMenu.Item onSelect={handleRestore}>
        <ArchiveRestoreIcon />
        Відновити
      </DropdownMenu.Item>

      <DropdownMenu.Separator />

      <DropdownMenu.Item
        variant="destructive"
        onSelect={() => (isDeleteDialogOpen = true)}
      >
        <TrashIcon />
        Видалити назавжди
      </DropdownMenu.Item>
    {:else}
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
        variant="warning"
        onSelect={() => (isArchiveDialogOpen = true)}
      >
        <ArchiveIcon />
        Архівувати
      </DropdownMenu.Item>
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>

<ClientMutationSheet {clientId} bind:open={isSheetOpen} />
<DeleteDialog
  bind:open={isArchiveDialogOpen}
  onDelete={handleArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description="Клієнт буде архівований разом з його автомобілями та замовленнями. Ви зможете відновити його пізніше."
/>
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  title="Видалити назавжди"
  confirmDisabled={deleteBlocked || deleteImpactQuery.isPending}
  description={deleteImpactQuery.isPending
    ? "Підрахунок пов'язаних даних…"
    : deleteBlocked
      ? "У клієнта є замовлення, тому видалити його не можна — разом з ним зникла б історія замовлень. Архівуйте клієнта: його можна буде відновити."
      : describeDeleteImpact(deleteImpactQuery.data)}
/>
