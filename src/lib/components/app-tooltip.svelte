<script lang="ts">
  import type { Snippet } from "svelte";

  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  import { cn } from "$lib/utils";

  type TriggerProps = Record<string, any>;

  let {
    trigger,
    content,
    heading = null,
    side = "top",
    align = "center",
    sideOffset = 6,
    delayDuration = 300,
    contentClass = "",
    hidden = undefined,
  }: {
    trigger: Snippet<[{ props: TriggerProps }]>;
    content: string | Snippet;
    /** Optional bold first line. */
    heading?: string | null;
    side?: "top" | "right" | "bottom" | "left";
    align?: "start" | "center" | "end";
    sideOffset?: number;
    delayDuration?: number;
    contentClass?: string;
    hidden?: boolean;
  } = $props();
</script>

<Tooltip.Root {delayDuration}>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      {@render trigger({ props: props as TriggerProps })}
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content
    {side}
    {align}
    {sideOffset}
    {hidden}
    class={cn(
      "z-floating flex w-fit max-w-xs flex-col gap-0.5 rounded-[8px] border border-border/70 bg-foreground px-3 py-2.5 text-[12.5px] leading-relaxed font-normal tracking-normal whitespace-normal text-background shadow-lg",
      contentClass,
    )}
  >
    {#if heading}
      <span class="block font-medium">{heading}</span>
    {/if}
    {#if typeof content === "string"}
      <span class={cn(heading && "block opacity-80")}>{content}</span>
    {:else}
      {@render content()}
    {/if}
  </Tooltip.Content>
</Tooltip.Root>
