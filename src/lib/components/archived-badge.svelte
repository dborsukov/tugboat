<script lang="ts">
  import ArchiveIcon from "@lucide/svelte/icons/archive";

  import { Badge } from "$lib/components/ui/badge";

  import { prettifyIsoDateString } from "$lib/tools";

  let { date, class: className }: { date?: string | null; class?: string } =
    $props();

  const formattedDate = $derived(
    date ? prettifyIsoDateString(date, "short") : null,
  );
</script>

<Badge
  variant="secondary"
  class="gap-1 px-2 py-0.5 text-xs font-normal text-muted-foreground shrink-0 {className}"
  title={formattedDate ? `Архівовано ${formattedDate}` : "Архівовано"}
>
  <ArchiveIcon class="size-3" />
  {#if formattedDate}
    В архіві з {formattedDate}
  {:else}
    В архіві
  {/if}
</Badge>
