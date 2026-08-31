<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";

  interface Props {
    open: boolean;
    title?: string;
    description?: string | null;
    confirmLabel?: string;
    actionVariant?: "default" | "destructive" | "warning";
    onDelete?: (() => void) | null;
  }

  let {
    open = $bindable(false),
    title = "Підтвердіть видалення",
    description,
    confirmLabel = "Видалити",
    actionVariant = "destructive",
    onDelete,
  }: Props = $props();
</script>

<AlertDialog.Root bind:open>
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
        onclick={() => {
          open = false;
          onDelete?.();
        }}>{confirmLabel}</AlertDialog.Action
      >
      <AlertDialog.Cancel variant="outline">Відмінити</AlertDialog.Cancel>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
