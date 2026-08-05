<script lang="ts">
  import RefreshIcon from "@lucide/svelte/icons/refresh-cw";

  import { cn } from "$lib/utils";

  import { Button } from "./ui/button";

  let {
    message,
    onRetry,
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
  <p class="text-md font-medium">
    Помилка завантаження: {message ?? "Невідома помилка"}
  </p>
  <Button
    variant="outline"
    class="hover:cursor-pointer text-md"
    size="lg"
    onclick={onRetry}
  >
    <RefreshIcon class="mr-1.5 size-4" />
    Спробувати знову
  </Button>
</div>
