<script lang="ts">
  import SheetHost from "$lib/components/sheet/sheet-host.svelte";

  import ClientSheetContent from "./client-sheet-content.svelte";

  let {
    clientId = null,
    open = $bindable(false),
    onCreated,
  }: {
    clientId?: number | null;
    open?: boolean;
    onCreated?: (id: number) => void;
  } = $props();
</script>

<SheetHost bind:open autofocus>
  <ClientSheetContent
    active={open}
    {clientId}
    onCancel={() => (open = false)}
    onCreated={(id) => {
      onCreated?.(id);
      open = false;
    }}
  />
</SheetHost>
