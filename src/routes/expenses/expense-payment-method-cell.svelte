<script lang="ts">
  import type { PaymentMethod } from "$lib/bindings";

  import { paymentMethodLabels } from "../work_orders/payment-method";

  let { method = null }: { method?: PaymentMethod | null } = $props();
</script>

{#if method}
  <p class="whitespace-nowrap">{paymentMethodLabels[method]}</p>
{:else}
  <p class="text-muted-foreground text-center">—</p>
{/if}
