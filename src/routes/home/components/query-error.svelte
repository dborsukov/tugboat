<script lang="ts">
  import RefreshIcon from "@lucide/svelte/icons/refresh-cw";

  import { Button } from "$lib/components/ui/button";

  import { cn } from "$lib/utils";

  let {
    error,
    onRetry,
    class: className,
  }: {
    error?: { message: string } | null;
    onRetry?: () => void;
    class?: string;
  } = $props();
</script>

<div class={cn("flex flex-col items-center gap-2 text-center", className)}>
  <p class="text-destructive text-sm" title={error?.message}>
    Не вдалося завантажити
  </p>
  {#if onRetry}
    <Button variant="outline" size="xs" onclick={onRetry}>
      <RefreshIcon />
      Спробувати ще
    </Button>
  {/if}
</div>
