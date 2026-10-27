<script lang="ts">
  import { createQuery, useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import CheckIcon from "@lucide/svelte/icons/check";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import UndoIcon from "@lucide/svelte/icons/undo-2";

  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner/index.js";

  import AppTooltip from "$lib/components/app-tooltip.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import type {
    AppSettings,
    DashboardModuleHeight,
    DashboardModuleSize,
  } from "$lib/bindings";
  import {
    settingsQueryOptions,
    settingsState,
  } from "$lib/states/settings.svelte";

  import ModuleFlow from "./components/module-flow.svelte";
  import ModulesSheet from "./components/modules-sheet.svelte";
  import MonthPicker from "./components/month-picker.svelte";
  import {
    isDefaultLayout,
    resolveLayout,
    toSavedLayout,
    visibleEntries,
    withHeight,
    withPosition,
    withWidth,
    type LayoutEntry,
  } from "./dashboard-layout";
  import { VAT_DUE_ID } from "./module-ids";
  import { monthKey, monthName } from "./period";
  import { createDashboardLayoutMutation } from "./queries";
  import { dashboardPeriod } from "./state.svelte";

  dashboardPeriod.start();

  const queryClient = useQueryClient();
  const settingsQuery = createQuery<AppSettings>(() => settingsQueryOptions());
  const dashboardLayout = createDashboardLayoutMutation(queryClient);

  /** Whether the saved arrangement is known yet. The page must not draw before it is.
      `resolveLayout` falls back to the shipped default arrangement while the settings are still arriving. */
  const layoutKnown = $derived(
    settingsQuery.isSuccess || settingsQuery.isError,
  );

  /** User defined layout, resolved against the registry. */
  const entries = $derived(
    resolveLayout(settingsState.settings?.dashboardLayout),
  );

  /** The saved value itself, which a save carries unknown modules through from. */
  const savedLayout = $derived(settingsState.settings?.dashboardLayout);

  const visible = $derived(
    visibleEntries(entries).filter(
      (entry) => entry.module.id !== VAT_DUE_ID || settingsState.isVatPayer,
    ),
  );

  let editing = $state(false);

  let confirmResetDialogOpen = $state(false);

  const reportsLabel = $derived.by(() => {
    const { year, month } = dashboardPeriod.anchor;
    const name = monthName({ year, month }, dashboardPeriod.timeZone);
    const sameYear = year === dashboardPeriod.currentAnchor.year;
    return `Звіти за ${name}${sameYear ? "" : ` ${year}`}`;
  });

  const defaultLayout = $derived(
    isDefaultLayout(settingsState.settings?.dashboardLayout),
  );

  function resetLayout() {
    dashboardLayout.mutate(
      toSavedLayout(resolveLayout(undefined), savedLayout),
    );
  }

  async function save(next: readonly LayoutEntry[]): Promise<unknown> {
    try {
      return await dashboardLayout.mutateAsync(
        toSavedLayout(next, savedLayout),
      );
    } catch {}
  }

  function setWidth(id: string, width: DashboardModuleSize) {
    const next = withWidth(entries, id, width);
    if (next !== entries) void save(next);
  }

  function setHeight(id: string, height: DashboardModuleHeight) {
    const next = withHeight(entries, id, height);
    if (next !== entries) void save(next);
  }

  /** A drop names a cell, so the module is put there and the rest make room. */
  function move(id: string, col: number, row: number): Promise<unknown> {
    const next = withPosition(entries, id, col, row);
    if (next === entries) return Promise.resolve(entries);

    return save(next);
  }

  // Escape leaves the layout editor. dnd-kit cancels an in-flight drag with the
  // same key and marks the event handled, so an escape that ended a drag does not
  // also close the editor.
  $effect(() => {
    if (!editing) return;

    const onKeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented) editing = false;
    };

    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
</script>

<PageLayout>
  {#snippet header()}
    <div class="relative h-full">
      <MonthPicker />

      <div class="absolute top-0 left-0 flex h-full items-center">
        <AppTooltip content={reportsLabel} side="bottom">
          {#snippet trigger({ props })}
            <Button
              {...props}
              variant="outline"
              size="icon-lg"
              class="lg:w-auto lg:gap-2.5 lg:px-4"
              aria-label={reportsLabel}
              onclick={() =>
                goto(`/reports?period=${monthKey(dashboardPeriod.anchor)}`)}
            >
              <FileSpreadsheetIcon />
              <span class="hidden whitespace-nowrap lg:inline"
                >{reportsLabel}</span
              >
            </Button>
          {/snippet}
        </AppTooltip>
      </div>

      <div class="absolute top-0 right-0 flex h-full items-center gap-2">
        {#if editing}
          <AppTooltip
            content={defaultLayout
              ? "Розташування вже стандартне"
              : "Скинути розташування"}
            side="bottom"
          >
            {#snippet trigger({ props })}
              <Button
                {...props}
                variant="outline"
                size="icon-lg"
                disabled={defaultLayout}
                aria-label="Скинути розташування"
                onclick={() => (confirmResetDialogOpen = true)}
              >
                <UndoIcon />
              </Button>
            {/snippet}
          </AppTooltip>
        {/if}

        <AppTooltip
          content={editing
            ? "Завершити редагування"
            : "Редагувати розташування"}
          side="bottom"
        >
          {#snippet trigger({ props })}
            <Button
              {...props}
              variant={editing ? "default" : "outline"}
              size="icon-lg"
              aria-pressed={editing}
              aria-label={editing
                ? "Завершити редагування розташування"
                : "Редагувати розташування"}
              onclick={() => (editing = !editing)}
            >
              {#if editing}<CheckIcon />{:else}<PencilIcon />{/if}
            </Button>
          {/snippet}
        </AppTooltip>

        <ModulesSheet />
      </div>
    </div>
  {/snippet}

  <AlertDialog.Root bind:open={confirmResetDialogOpen}>
    <AlertDialog.Content>
      <AlertDialog.Header>
        <AlertDialog.Title>Скинути розташування?</AlertDialog.Title>
        <AlertDialog.Description>
          Розташування, розміри, видимість і налаштування модулів повернуться до
          стандартних.
        </AlertDialog.Description>
      </AlertDialog.Header>
      <AlertDialog.Footer>
        <AlertDialog.Cancel>Скасувати</AlertDialog.Cancel>
        <AlertDialog.Action
          onclick={() => {
            resetLayout();
            confirmResetDialogOpen = false;
          }}>Скинути</AlertDialog.Action
        >
      </AlertDialog.Footer>
    </AlertDialog.Content>
  </AlertDialog.Root>

  {#if editing}
    <p class="text-muted-foreground mt-4 text-sm">
      Перетягніть модуль за ручку, щоб поставити його в потрібне місце сітки,
      або виберіть ширину чи висоту.
    </p>
  {/if}

  {#if layoutKnown && visible.length > 0}
    <ModuleFlow
      entries={visible}
      {editing}
      onMove={move}
      onWidth={setWidth}
      onHeight={setHeight}
    />
  {/if}

  {#if !layoutKnown}
    <div class="mt-4 flex flex-col items-center gap-2" role="status">
      <Spinner class="text-muted-foreground size-6" />
      <p class="text-muted-foreground text-sm">Завантаження розташування…</p>
    </div>
  {/if}

  {#if layoutKnown && visible.length === 0}
    <p class="text-muted-foreground mt-8 text-center text-sm">
      Усі модулі вимкнено. Увімкніть потрібні у панелі «Модулі головної».
    </p>
  {/if}
</PageLayout>

<style>
  /* No sideways scrollbar while a module is being dragged. */
  :global(.overflow-auto:has([data-dragging])) {
    overflow-x: hidden;
  }
</style>
