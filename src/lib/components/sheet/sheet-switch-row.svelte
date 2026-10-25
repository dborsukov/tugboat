<script lang="ts">
  import type { Snippet } from "svelte";

  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";

  import { cn } from "$lib/utils";

  let {
    id,
    checked,
    onCheckedChange,
    label,
    switchProps = {},
    class: className,
    children,
  }: {
    id: string;
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
    label: string;
    switchProps?: Record<string, unknown>;
    class?: string;
    children?: Snippet;
  } = $props();

  const controlId = $derived(
    typeof switchProps.id === "string" ? switchProps.id : id,
  );
</script>

<div
  class={cn(
    "flex items-start justify-between gap-4 rounded-[10px] border border-border bg-card px-3 py-3",
    className,
  )}
>
  <div class="min-w-0 flex-1 space-y-1">
    <Label for={controlId} class="text-[13px] leading-snug font-medium">
      {label}
    </Label>
    {#if children}
      <div class="text-[11.5px] leading-relaxed text-muted-foreground">
        {@render children()}
      </div>
    {/if}
  </div>
  <Switch
    {...switchProps}
    id={controlId}
    {checked}
    class="mt-0.5"
    onCheckedChange={(value) => onCheckedChange(value === true)}
  />
</div>
