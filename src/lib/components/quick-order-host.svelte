<script lang="ts">
  import { quickOrder } from "$lib/states/quick-order.svelte";

  import QuickOrderFlow from "../../routes/work_orders/quick-order-flow.svelte";

  function isTextTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target.isContentEditable ||
      target.closest('[contenteditable="true"]') != null
    );
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.defaultPrevented) return;
    if (!(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) {
      return;
    }
    const isN = event.code === "KeyN" || event.key.toLowerCase() === "n";
    if (!isN) return;
    if (isTextTarget(event.target)) return;

    // Never stack the quick-order sheet on top of another modal.
    if (
      !quickOrder.open &&
      document.querySelector(
        '[data-slot="dialog-content"], [data-slot="alert-dialog-content"], [data-slot="sheet-content"]',
      )
    ) {
      return;
    }

    event.preventDefault();
    quickOrder.openFromCurrentRoute();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<QuickOrderFlow
  bind:open={quickOrder.open}
  context={quickOrder.context}
  requestId={quickOrder.requestId}
/>
