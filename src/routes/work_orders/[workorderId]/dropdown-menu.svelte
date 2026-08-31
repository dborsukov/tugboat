<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import type { WorkOrderDeleteImpact } from "$lib/bindings";
  import { pluralizeUk } from "$lib/tools";

  import {
    createArchiveWorkOrderMutation,
    createDeleteWorkOrderMutation,
    createRestoreWorkOrderMutation,
    createWorkOrderDeleteImpactQuery,
  } from "../queries";
  import WorkOrderMutationSheet from "../work-order-mutation-sheet.svelte";

  let {
    workOrderId,
    isLocked = false,
    isArchived = false,
  }: {
    workOrderId: string;
    isLocked?: boolean;
    isArchived?: boolean;
  } = $props();
  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);
  let isArchiveDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const deleteWorkOrderMutation = createDeleteWorkOrderMutation(queryClient);
  const archiveWorkOrderMutation = createArchiveWorkOrderMutation(queryClient);
  const restoreWorkOrderMutation = createRestoreWorkOrderMutation(queryClient);

  const deleteImpactQuery = createWorkOrderDeleteImpactQuery(
    () => workOrderId,
    () => isDeleteDialogOpen,
  );

  function describeDeleteImpact(impact: WorkOrderDeleteImpact | undefined) {
    if (!impact) {
      return "Цю дію неможливо відмінити. Замовлення буде видалено безповоротно разом з усіма пов'язаними даними.";
    }
    const parts: string[] = [];
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
      return "Цю дію неможливо відмінити. Замовлення буде видалено безповоротно.";
    }
    return `Буде видалено: ${parts.join(", ")}. Цю дію неможливо відмінити.`;
  }

  function handleDelete() {
    deleteWorkOrderMutation.mutate(workOrderId, {
      onSuccess: async () => {
        isDeleteDialogOpen = false;
        await goto("/work_orders");
      },
    });
  }

  function handleArchive() {
    archiveWorkOrderMutation.mutate(workOrderId);
  }

  function handleRestore() {
    restoreWorkOrderMutation.mutate(workOrderId);
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon-lg" })}
  >
    <EllipsisIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
    <DropdownMenu.Group>
      {#if isArchived}
        <DropdownMenu.Item onSelect={handleRestore}>
          <ArchiveRestoreIcon />
          Відновити
        </DropdownMenu.Item>
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
        <DropdownMenu.Item
          variant="warning"
          onSelect={() => (isArchiveDialogOpen = true)}
        >
          <ArchiveIcon />
          Архівувати
        </DropdownMenu.Item>
      {/if}
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
  bind:open={isArchiveDialogOpen}
  onDelete={handleArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description="Замовлення буде архівоване. Ви зможете відновити його пізніше."
/>
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  title="Видалити назавжди"
  description={deleteImpactQuery.isPending
    ? "Підрахунок пов'язаних даних…"
    : describeDeleteImpact(deleteImpactQuery.data)}
/>
