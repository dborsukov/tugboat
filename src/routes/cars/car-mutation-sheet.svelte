<script lang="ts">
  import SheetHost from "$lib/components/sheet/sheet-host.svelte";

  import CarSheetContent from "./car-sheet-content.svelte";

  let {
    carId = null,
    initClientId = null,
    open = $bindable(false),
    onCreated,
  }: {
    carId?: number | null;
    initClientId?: number | null;
    open?: boolean;
    onCreated?: (id: number) => void;
  } = $props();
</script>

<SheetHost bind:open autofocus>
  <CarSheetContent
    active={open}
    {carId}
    ownerClientId={initClientId}
    onCancel={() => (open = false)}
    onCreated={(id) => {
      onCreated?.(id);
      open = false;
    }}
  />
</SheetHost>
