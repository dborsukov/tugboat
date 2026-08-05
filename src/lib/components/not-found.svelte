<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    message,
    class: className,
    ...props
  }: { message?: String; onRetry?: () => void; class?: String } = $props();
</script>

<div
  class={cn(
    "flex flex-col items-center justify-center gap-3 text-destructive",
    className,
  )}
  {...props}
>
  <div class="flex gap-2">
    <p class="text-lg font-medium">
      {message ?? "Не знайдено"}
    </p>
  </div>
</div>
