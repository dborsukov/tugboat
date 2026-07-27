<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    description,
    children,
  }: { label: string; description?: string; children: Snippet } = $props();
</script>

<div class="settings-row">
  <div class="settings-row-text">
    <span class="settings-row-label">{label}</span>
    {#if description}
      <span class="settings-row-description">{description}</span>
    {/if}
  </div>
  <div class="settings-row-control">
    {@render children()}
  </div>
</div>

<style>
  .settings-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    padding: 1rem 0;
    border-bottom: 1px solid var(--border, #e5e7eb);
  }

  .settings-row:last-child {
    border-bottom: none;
  }

  .settings-row-text {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .settings-row-label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  .settings-row-description {
    font-size: 0.8125rem;
    color: var(--muted-foreground, #6b7280);
  }

  .settings-row-control {
    flex-shrink: 0;
  }
</style>
