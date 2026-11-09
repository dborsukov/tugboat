<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";

  interface Props {
    open: boolean;
    title?: string;
    description?: string | null;
    confirmLabel?: string;
    /** Set when the action is not available, the description should mention why. */
    confirmDisabled?: boolean;
    actionVariant?: "default" | "destructive" | "warning";
    onDelete?: (() => void) | null;
    onOpenChange?: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    title = "Підтвердіть видалення",
    description,
    confirmLabel = "Видалити",
    confirmDisabled = false,
    actionVariant = "destructive",
    onDelete,
    onOpenChange,
  }: Props = $props();
</script>

<AlertDialog.Root
  bind:open
  onOpenChange={(value) => {
    if (!value) onOpenChange?.(false);
  }}
>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>{title}</AlertDialog.Title>
      {#if description != null}
        <AlertDialog.Description>
          {description}
        </AlertDialog.Description>
      {/if}
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Action
        variant={actionVariant}
        disabled={confirmDisabled}
        onclick={() => {
          if (confirmDisabled) return;
          open = false;
          onDelete?.();
        }}>{confirmLabel}</AlertDialog.Action
      >
      <AlertDialog.Cancel variant="outline">Відмінити</AlertDialog.Cancel>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
