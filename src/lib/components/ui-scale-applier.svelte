<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  import { settingsState } from "$lib/states/settings.svelte";

  // Apply the persisted UI scale (webview zoom) as soon as settings load, and
  // re-apply automatically when the scale changes in settings
  $effect(() => {
    const uiScale = settingsState.settings?.uiScale;
    if (uiScale === undefined) return;
    getCurrentWebviewWindow()
      .setZoom(uiScale)
      .catch(() => {});
  });
</script>
