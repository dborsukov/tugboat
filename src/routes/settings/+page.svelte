<script lang="ts">
  import { resetMode, setMode } from "mode-watcher";
  import { toast } from "svelte-sonner";

  import SaveIcon from "@lucide/svelte/icons/save";
  import DiscardIcon from "@lucide/svelte/icons/x";

  import { commands, type ThemeMode } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Spinner } from "$lib/components/ui/spinner";

  import PageLayout from "$lib/components/page-layout.svelte";

  import SettingsRow from "./settings-row.svelte";

  let saving = $state(false);
  let draftSettings = $state($state.snapshot(appStore.settings));
  let hasChanges = $derived(
    JSON.stringify(draftSettings) !== JSON.stringify(appStore.settings),
  );

  const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
    { value: "system", label: "Системна" },
    { value: "light", label: "Світла" },
    { value: "dark", label: "Темна" },
  ];

  const themeLabel = $derived(
    THEME_OPTIONS.find((o) => o.value === draftSettings.theme)?.label ??
      "Оберіть тему",
  );

  async function save() {
    saving = true;
    let res = await commands.updateSettings(draftSettings);
    if (res.status === "ok") {
      appStore.updateSettings(res.data);
      res.data.theme === "system" ? resetMode() : setMode(res.data.theme);
      toast.success("Налаштування збережено", { position: "bottom-right" });
    } else {
      toast.error("Виникла помилка", {
        description: res.error.message,
        position: "bottom-right",
      });
    }
    saving = false;
  }

  function discard() {
    draftSettings = $state.snapshot(appStore.settings);
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full flex items-center justify-end">
      <ButtonGroup.Root>
        <Button
          disabled={!hasChanges || saving}
          variant="outline"
          size="icon"
          onclick={discard}
        >
          <DiscardIcon />
        </Button>
        <Button
          disabled={!hasChanges || saving}
          variant="outline"
          onclick={save}
        >
          {#if saving}
            <Spinner />
          {:else}
            <SaveIcon />Зберегти
          {/if}
        </Button>
      </ButtonGroup.Root>
    </div>
  {/snippet}
  <div class="p-4 flex flex-col gap-6">
    <Card.Root>
      <Card.Header>
        <Card.Title>Оформлення</Card.Title>
      </Card.Header>
      <Card.Content>
        <SettingsRow label="Тема" description="Як має виглядати застосунок.">
          <Select.Root type="single" bind:value={draftSettings.theme}>
            <Select.Trigger class="w-30">
              {themeLabel}
            </Select.Trigger>
            <Select.Content>
              {#each THEME_OPTIONS as option (option.value)}
                <Select.Item value={option.value} label={option.label}>
                  {option.label}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </SettingsRow>
      </Card.Content>
    </Card.Root>
  </div>
</PageLayout>
