<script lang="ts">
  import { Button } from "$lib/components/ui/button";

  import { cn } from "$lib/utils";

  import ClientAvatar from "../../../routes/clients/client-avatar.svelte";

  let {
    name,
    hint,
    onEdit,
    editLabel = "Змінити",
    class: className,
  }: {
    name: string;
    hint?: string;
    onEdit?: () => void;
    editLabel?: string;
    class?: string;
  } = $props();
</script>

<div
  class={cn(
    "flex min-h-12 items-center gap-3 rounded-[10px] border border-border bg-card px-3 py-2",
    className,
  )}
>
  <ClientAvatar {name} class="size-8 shrink-0" />
  <div class="min-w-0 flex-1">
    <p class="truncate text-sm font-semibold">{name}</p>
    {#if hint}
      <p class="truncate text-xs text-muted-foreground">{hint}</p>
    {/if}
  </div>
  {#if onEdit}
    <Button
      type="button"
      variant="ghost"
      size="xs"
      class="h-7 shrink-0 rounded-[6px] px-2 text-xs font-medium text-brand hover:bg-brand/10 hover:text-brand"
      onclick={onEdit}
    >
      {editLabel}
    </Button>
  {/if}
</div>
