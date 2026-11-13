<script lang="ts">
  import { Kbd, KbdGroup } from "$lib/components/ui/kbd";

  import { isMacOS } from "$lib/platform";

  let { keys, class: className = "" }: { keys: string[]; class?: string } =
    $props();

  // macOS convention is ⌘ with no plus signs between the keys.
  const labels = $derived(
    keys.map((key) => (key === "Ctrl" && isMacOS ? "⌘" : key)),
  );
</script>

<KbdGroup class="ml-auto {className}">
  {#each labels as label, index (index)}
    {#if index > 0 && !isMacOS}
      <span class="text-xs text-muted-foreground">+</span>
    {/if}
    <Kbd>{label}</Kbd>
  {/each}
</KbdGroup>
