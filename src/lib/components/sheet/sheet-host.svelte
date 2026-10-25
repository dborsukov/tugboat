<script lang="ts">
  import type { Snippet } from "svelte";

  import * as Sheet from "$lib/components/ui/sheet";

  import { cn } from "$lib/utils";

  let {
    open = $bindable(false),
    autofocus = false,
    class: className,
    children,
  }: {
    open?: boolean;
    autofocus?: boolean;
    class?: string;
    children: Snippet;
  } = $props();
</script>

<Sheet.Root bind:open>
  <Sheet.Content
    side="right"
    showCloseButton={false}
    class={cn(
      "w-full gap-0 overflow-hidden p-0 data-[side=right]:w-full sm:max-w-[480px] sm:data-[side=right]:max-w-[480px]",
      className,
    )}
    onEscapeKeydown={(event) => event.preventDefault()}
    onInteractOutside={(event) => event.preventDefault()}
    onOpenAutoFocus={(event) => {
      if (!autofocus) return;
      event.preventDefault();
      const content = event.currentTarget as HTMLElement | null;
      let attempts = 0;

      const focusTarget = () => {
        const target = [
          ...(content?.querySelectorAll<HTMLElement>(
            "[data-sheet-autofocus]",
          ) ?? []),
        ].find((element) => element.getClientRects().length > 0);
        if (target) {
          target.focus();
          return;
        }
        if (++attempts < 12) setTimeout(focusTarget, 40);
      };

      focusTarget();
    }}
  >
    {@render children()}
  </Sheet.Content>
</Sheet.Root>
