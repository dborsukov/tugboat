<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { resetMode, setMode } from "mode-watcher";

  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Item from "$lib/components/ui/item/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { type SidebarMetric, type ThemeMode } from "$lib/bindings";

  import {
    createUpdateSidebarMetricMutation,
    createUpdateSidebarRecentItemsMutation,
    createUpdateThemeMutation,
    createUpdateUiScaleMutation,
  } from "../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../settings-context";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateThemeMutation = createUpdateThemeMutation(queryClient);
  const updateUiScaleMutation = createUpdateUiScaleMutation(queryClient);
  const updateSidebarMetricMutation =
    createUpdateSidebarMetricMutation(queryClient);
  const updateSidebarRecentItemsMutation =
    createUpdateSidebarRecentItemsMutation(queryClient);

  // ---------- Theme ----------

  const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
    { value: "system", label: "Системна" },
    { value: "light", label: "Світла" },
    { value: "dark", label: "Темна" },
  ];

  const themeLabel = $derived(
    THEME_OPTIONS.find((o) => o.value === draft?.theme)?.label ??
      "Оберіть тему",
  );

  function onThemeChange(value: string) {
    if (!draft) return;
    draft.theme = value as ThemeMode;
    draft.theme === "system" ? resetMode() : setMode(draft.theme);
    updateThemeMutation.mutate(draft.theme);
  }

  // ---------- UI scale (webview zoom) ----------

  const uiScalePercent = $derived(
    draft ? Math.round(draft.uiScale * 100) : 100,
  );

  function onUiScaleChange(value: number) {
    if (!draft) return;
    draft.uiScale = value;
  }

  function onUiScaleCommit(value: number) {
    if (!draft) return;
    draft.uiScale = value;
    getCurrentWebviewWindow()
      .setZoom(value)
      .catch(() => {});
    // fires once per slider release, so it can persist immediately
    updateUiScaleMutation.mutate(draft.uiScale);
  }

  function resetUiScale() {
    onUiScaleCommit(1.0);
  }

  // ---------- Sidebar ----------

  const SIDEBAR_METRIC_OPTIONS: { value: SidebarMetric; label: string }[] = [
    { value: "revenue", label: "Виручка" },
    { value: "gross_income", label: "Валовий дохід" },
    { value: "financial_result", label: "Фінансовий результат" },
  ];

  const sidebarMetricLabel = $derived(
    SIDEBAR_METRIC_OPTIONS.find(
      (option) => option.value === draft?.sidebarMetric,
    )?.label ?? "Фінансовий результат",
  );

  function onSidebarMetricChange(value: string) {
    if (!draft) return;
    const metric = value as SidebarMetric;
    draft.sidebarMetric = metric;
    updateSidebarMetricMutation.mutate(metric);
  }

  function onShowRecentItemsChange(checked: boolean) {
    if (!draft) return;
    draft.showRecentItems = checked;
    updateSidebarRecentItemsMutation.mutate(checked);
  }
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 border border-border rounded bg-card">
    <Item.Root id="settings-theme">
      <Item.Content>
        <Item.Title>Тема</Item.Title>
        <Item.Description>Тема оформлення застосунку</Item.Description>
        {#if updateThemeMutation.isError}
          <p class="text-xs text-destructive mt-1">
            Не вдалося зберегти тему: {updateThemeMutation.error?.message}
          </p>
        {/if}
      </Item.Content>
      <Item.Actions>
        <Select.Root
          type="single"
          value={draft?.theme}
          onValueChange={onThemeChange}
        >
          <Select.Trigger class="w-30">{themeLabel}</Select.Trigger>
          <Select.Content>
            {#each THEME_OPTIONS as option (option.value)}
              <Select.Item value={option.value} label={option.label}>
                {option.label}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </Item.Actions>
    </Item.Root>

    <Item.Root id="settings-ui-scale">
      <Item.Content>
        <Item.Title>Масштаб інтерфейсу</Item.Title>
        <Item.Description
          >Розмір тексту та елементів інтерфейсу</Item.Description
        >
        {#if updateUiScaleMutation.isError}
          <p class="text-xs text-destructive mt-1">
            Не вдалося зберегти масштаб: {updateUiScaleMutation.error?.message}
          </p>
        {/if}
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-1">
            <Button variant="ghost" size="icon" onclick={resetUiScale}>
              <RotateCcwIcon class="text-muted-foreground" />
            </Button>
            <p>{uiScalePercent}%</p>
          </div>
          <Slider
            type="single"
            value={draft?.uiScale}
            onValueChange={onUiScaleChange}
            onValueCommit={onUiScaleCommit}
            min={0.5}
            max={2.0}
            step={0.05}
            class="w-40"
          />
        </div>
      </Item.Actions>
    </Item.Root>
  </div>

  <p class="my-4 px-4 text-lg font-semibold">Бічна панель</p>

  <div class="overflow-hidden border border-border rounded bg-card">
    <Item.Root id="settings-sidebar-metric">
      <Item.Content>
        <Item.Title>Метрика на кнопці головної сторінки</Item.Title>
        <Item.Description
          >Оберіть метрику що буде відображатись поруч з місяцем на кнопці
          головної сторінки</Item.Description
        >
        {#if updateSidebarMetricMutation.isError}
          <p class="text-xs text-destructive mt-1">
            Не вдалося зберегти показник: {updateSidebarMetricMutation.error
              ?.message}
          </p>
        {/if}
      </Item.Content>
      <Item.Actions>
        <Select.Root
          type="single"
          value={draft?.sidebarMetric ?? "financial_result"}
          onValueChange={onSidebarMetricChange}
        >
          <Select.Trigger class="w-48">{sidebarMetricLabel}</Select.Trigger>
          <Select.Content>
            {#each SIDEBAR_METRIC_OPTIONS as option (option.value)}
              <Select.Item value={option.value} label={option.label}>
                {option.label}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </Item.Actions>
    </Item.Root>

    <Item.Root id="settings-sidebar-recents">
      <Item.Content>
        <Item.Title>Нещодавні</Item.Title>
        <Item.Description
          >Чи відображати в бічній панелі секцію нещодавно відвіданих сторінок</Item.Description
        >
        {#if updateSidebarRecentItemsMutation.isError}
          <p class="text-xs text-destructive mt-1">
            Не вдалося зберегти налаштування: {updateSidebarRecentItemsMutation
              .error?.message}
          </p>
        {/if}
      </Item.Content>
      <Item.Actions>
        <Switch
          checked={draft?.showRecentItems ?? true}
          onCheckedChange={onShowRecentItemsChange}
          aria-label="Показувати нещодавні"
        />
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
