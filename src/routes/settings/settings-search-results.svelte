<script lang="ts">
  import { settingsSearch } from "./settings-search.svelte";
</script>

{#if settingsSearch.query.trim()}
  <div class="mb-1 border-b border-border px-2 pb-2">
    {#if settingsSearch.results.length === 0}
      <p class="px-2 py-3 text-center text-sm text-muted-foreground">
        Нічого не знайдено
      </p>
    {:else}
      <div class="flex flex-col gap-0.5">
        {#each settingsSearch.results as item (item.id)}
          <button
            type="button"
            class="flex w-full flex-col items-start gap-0.5 rounded-lg px-2 py-1.5 text-left text-sm transition-colors hover:cursor-pointer hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onclick={() => settingsSearch.select(item)}
          >
            <span class="font-medium">{item.label}</span>
            {#if item.hint}
              <span class="text-xs text-muted-foreground">{item.hint}</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}
