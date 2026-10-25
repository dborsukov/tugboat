<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    text,
    query,
    class: className,
    muted = false,
  }: {
    text: string;
    query?: string;
    class?: string;
    muted?: boolean;
  } = $props();

  function escapeRegExp(value: string) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }

  const parts = $derived.by(() => {
    const words = [
      ...new Set(
        (query ?? "")
          .trim()
          .split(/\s+/)
          .filter((word) => word.length > 0),
      ),
    ];
    if (words.length === 0) return [{ text, match: false }];

    const pattern = new RegExp(`(${words.map(escapeRegExp).join("|")})`, "gi");
    const testPattern = new RegExp(
      `^(${words.map(escapeRegExp).join("|")})$`,
      "i",
    );
    return text
      .split(pattern)
      .filter((part) => part.length > 0)
      .map((part) => ({ text: part, match: testPattern.test(part) }));
  });
</script>

<span class={cn(muted && "text-muted-foreground", className)}>
  {#each parts as part}
    {#if part.match}
      <mark
        class="rounded-[2px] bg-brand/20 px-0.5 font-semibold text-foreground"
        >{part.text}</mark
      >
    {:else}
      {part.text}
    {/if}
  {/each}
</span>
