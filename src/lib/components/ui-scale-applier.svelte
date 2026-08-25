<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  import { createSettingsQuery } from "../../routes/settings/queries";

  const settingsQuery = createSettingsQuery();

  // Apply the persisted UI scale (webview zoom) as soon as settings load, and
  // re-apply automatically when the scale changes in settings
  $effect(() => {
    const uiScale = settingsQuery.data?.uiScale;
    if (uiScale === undefined) return;
    getCurrentWebviewWindow()
      .setZoom(uiScale)
      .catch(() => {});
  });
</script>
