<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import FileCheckIcon from "@lucide/svelte/icons/file-check-2";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import ShieldIcon from "@lucide/svelte/icons/shield-check";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import type {
    DocumentKind,
    WorkOrderDeleteImpact,
    WorkOrderType,
  } from "$lib/bindings";
  import { pluralizeUk } from "$lib/tools";

  import {
    createArchiveWorkOrderMutation,
    createDeleteWorkOrderMutation,
    createRestoreWorkOrderMutation,
    createWorkOrderDeleteImpactQuery,
  } from "../queries";
  import WorkOrderMutationSheet from "../work-order-mutation-sheet.svelte";
  import WarrantyOrderCreateDialog from "./warranty-order-create-dialog.svelte";

  let {
    workOrderId,
    type = "Regular",
    odometerIn = 0,
    isLocked = $bindable(false),
    isArchived = false,
    onGenerate,
  }: {
    workOrderId: string;
    type?: WorkOrderType;
    odometerIn?: number;
    isLocked?: boolean;
    isArchived?: boolean;
    onGenerate: (kind: DocumentKind) => void;
  } = $props();
  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);
  let isArchiveDialogOpen = $state(false);
  let isWarrantyCreateOpen = $state(false);

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

  function describeDeleteBlocked(impact: WorkOrderDeleteImpact | undefined) {
    const claims = impact?.warrantyClaimCount ?? 0;
    if (claims > 0) {
      return `Видалення неможливе: із замовленням пов'язано ${claims} ${pluralizeUk(
        claims,
        [
          "гарантійне замовлення",
          "гарантійні замовлення",
          "гарантійних замовлень",
        ],
      )}. Спочатку видаліть їх.`;
    }

    const documents = impact?.documentCount ?? 0;
    if (documents > 0) {
      return `Видалення неможливе: за замовленням створено ${documents} ${pluralizeUk(
        documents,
        ["документ", "документи", "документів"],
      )}. Архівуйте замовлення, щоб зберегти їх.`;
    }

    if (impact?.isPaid) {
      return "Видалення неможливе: замовлення оплачене. Архівуйте його, щоб зберегти історію.";
    }

    return null;
  }

  const deleteBlocked = $derived(
    (deleteImpactQuery.data?.warrantyClaimCount ?? 0) > 0 ||
      (deleteImpactQuery.data?.documentCount ?? 0) > 0 ||
      (deleteImpactQuery.data?.isPaid ?? false),
  );

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
        <DropdownMenu.Separator />
        <DropdownMenu.Label>Створити документ</DropdownMenu.Label>
        <DropdownMenu.Item onclick={() => onGenerate("naryad_zamovlennya")}>
          <FileTextIcon />
          Наряд-замовлення
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={() => onGenerate("akt_vykonanykh_robit")}>
          <FileCheckIcon />
          Акт виконаних робіт
        </DropdownMenu.Item>
        {#if type === "Regular"}
          <DropdownMenu.Separator />
          <DropdownMenu.Item onSelect={() => (isWarrantyCreateOpen = true)}>
            <ShieldIcon />
            Оформити гарантійний випадок
          </DropdownMenu.Item>
        {/if}
        <DropdownMenu.Separator />
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
<WarrantyOrderCreateDialog
  bind:open={isWarrantyCreateOpen}
  sourceWorkOrderId={type === "Regular" ? workOrderId : null}
  {odometerIn}
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
  confirmDisabled={deleteBlocked || deleteImpactQuery.isPending}
  description={deleteImpactQuery.isPending
    ? "Підрахунок пов'язаних даних…"
    : (describeDeleteBlocked(deleteImpactQuery.data) ??
      describeDeleteImpact(deleteImpactQuery.data))}
/>
