<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";

  interface Props {
    open: boolean;
    description?: string | null;
    onDelete?: (() => void) | null;
  }

  let { open = $bindable(false), description, onDelete }: Props = $props();
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Підтвердіть видалення</AlertDialog.Title>
      {#if description != null}
        <AlertDialog.Description>
          {description}
        </AlertDialog.Description>
      {/if}
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Action variant="destructive" onclick={onDelete}
        >Видалити</AlertDialog.Action
      >
      <AlertDialog.Cancel variant="outline">Відмінити</AlertDialog.Cancel>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
