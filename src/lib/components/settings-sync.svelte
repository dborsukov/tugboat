<script lang="ts">
  import {
    createSettingsQuery,
    settingsState,
  } from "$lib/states/settings.svelte";

  const settingsQuery = createSettingsQuery();

  $effect(() => {
    settingsState.settings = settingsQuery.data ?? null;
  });
</script>
