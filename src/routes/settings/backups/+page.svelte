<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { message, open } from "@tauri-apps/plugin-dialog";
  import { exit, relaunch } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import { dev } from "$app/environment";

  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
  import ClockIcon from "@lucide/svelte/icons/clock";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import OctagonXIcon from "@lucide/svelte/icons/octagon-x";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";

  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import { Badge } from "$lib/components/ui/badge";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import * as Item from "$lib/components/ui/item/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Switch } from "$lib/components/ui/switch";

  import AppTooltip from "$lib/components/app-tooltip.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { commands } from "$lib/bindings";
  import type {
    BackupInfo,
    BackupKind,
    BackupOverview,
    BackupSettings,
  } from "$lib/bindings";
  import { isMacOS, isWindows } from "$lib/platform";
  import {
    backupAttention,
    STALE_AFTER_DAYS,
  } from "$lib/states/backup-attention.svelte";
  import { settingsState } from "$lib/states/settings.svelte";
  import { isoDayOf } from "$lib/tools";
  import { cn } from "$lib/utils";

  import { SETTINGS_CONTENT_WIDTH_CLASS } from "../settings-context";

  const STALE_AFTER_MS = STALE_AFTER_DAYS * 24 * 60 * 60 * 1000;

  const INTERVAL_OPTIONS: { value: number; label: string }[] = [
    { value: 6, label: "Кожні 6 годин" },
    { value: 12, label: "Кожні 12 годин" },
    { value: 24, label: "Щодня" },
    { value: 48, label: "Кожні 2 дні" },
    { value: 168, label: "Щотижня" },
  ];

  const RETENTION_OPTIONS: { value: number; label: string }[] = [
    { value: 5, label: "5" },
    { value: 10, label: "10" },
    { value: 14, label: "14" },
    { value: 30, label: "30" },
  ];

  const KIND_LABELS: Record<BackupKind, string> = {
    manual: "Вручну",
    scheduled: "Автоматично",
    preMigration: "Автоматично",
    preUpdate: "Автоматично",
    preRestore: "Перед відновленням",
  };

  const OPEN_FOLDER_LABEL = isMacOS
    ? "Відкрити у Finder"
    : isWindows
      ? "Відкрити у провіднику"
      : "Відкрити у файловому менеджері";

  type RestoreTarget = {
    info: BackupInfo;
    fileName?: string;
    path?: string;
  };

  type StatusState = "ok" | "running" | "stale" | "failed" | "empty";

  let overview = $state<BackupOverview | null>(null);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);
  let creating = $state(false);
  let restoring = $state(false);
  let showAll = $state(false);

  let restoreTarget = $state<RestoreTarget | null>(null);
  let restoreOpen = $state(false);

  const settings = $derived(overview?.settings ?? null);
  const backups = $derived(overview?.backups ?? []);
  const status = $derived(overview?.status ?? null);
  const folderAvailable = $derived(overview?.folderAvailable ?? true);
  const latest = $derived(backups[0] ?? null);
  const running = $derived(creating || (status?.running ?? false));

  const folderPath = $derived(
    settings?.folder ?? overview?.effectiveFolder ?? "",
  );

  const intervalLabel = $derived(
    INTERVAL_OPTIONS.find((option) => option.value === settings?.intervalHours)
      ?.label ?? "Щодня",
  );

  const statusState = $derived.by<StatusState>(() => {
    if (running) return "running";
    if (!folderAvailable || status?.lastError) return "failed";
    if (!latest) return "empty";

    const created = new Date(latest.createdAt).getTime();
    if (Number.isFinite(created) && Date.now() - created > STALE_AFTER_MS) {
      return "stale";
    }
    return "ok";
  });

  const statusHeadline = $derived.by(() => {
    if (statusState === "running") return "Створення копії…";
    if (statusState === "failed") return "Не вдалося створити копію";
    if (statusState === "empty") return "Копій ще немає";
    if (statusState === "stale" && latest) {
      return `Остання копія: ${relativeTime(latest.createdAt)}`;
    }
    if (statusState === "ok" && latest) {
      return `Остання копія: ${relativeTime(latest.createdAt)}`;
    }
    return "";
  });

  const statusDetail = $derived.by(() => {
    if (statusState === "running") return "Не закривайте застосунок";
    if (statusState === "failed") {
      if (!folderAvailable) {
        return "Тека недоступна. Перевірте, чи підключено диск або хмарну папку.";
      }
      return status?.lastError ?? "Не вдалося створити копію";
    }
    if (statusState === "empty")
      return "Створіть першу копію, щоб захистити дані";
    if (statusState === "stale") {
      return settings?.enabled === false ? "Автоматичні копії вимкнено" : null;
    }
    if (statusState === "ok" && latest) {
      const kind = latest.kind === "manual" ? "Вручну" : "Автоматично";
      return `${formatBytes(latest.sizeBytes)} · версія ${latest.appVersion} · ${kind}`;
    }
    return null;
  });

  const historyRows = $derived(showAll ? backups : backups.slice(0, 5));

  function timeZone(): string | undefined {
    return settingsState.settings?.timezone ?? undefined;
  }

  function formatHistoryDate(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;

    return new Intl.DateTimeFormat("uk-UA", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      hourCycle: "h23",
      timeZone: timeZone(),
    })
      .format(date)
      .replace(/\s*р\./g, "");
  }

  function relativeTime(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;

    const time = new Intl.DateTimeFormat("uk-UA", {
      hour: "2-digit",
      minute: "2-digit",
      hourCycle: "h23",
      timeZone: timeZone(),
    }).format(date);

    const today = isoDayOf(new Date().toISOString());
    const day = isoDayOf(value);
    const days = Math.round(
      (Date.parse(`${today}T00:00:00Z`) - Date.parse(`${day}T00:00:00Z`)) /
        86_400_000,
    );

    if (days <= 0) return `сьогодні, ${time}`;
    if (days === 1) return `вчора, ${time}`;
    return `${days} днів тому`;
  }

  function formatBytes(value: number | null): string {
    if (!value || value <= 0) return "—";
    const units = ["Б", "КБ", "МБ", "ГБ"];
    let size = value;
    let unit = 0;
    while (size >= 1024 && unit < units.length - 1) {
      size /= 1024;
      unit += 1;
    }
    return `${size < 10 ? size.toFixed(1) : Math.round(size)} ${units[unit]}`;
  }

  function copiesLabel(count: number): string {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return `${count} копія`;
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 10 || mod100 >= 20)) {
      return `${count} копії`;
    }
    return `${count} копій`;
  }

  function middleTruncate(value: string, max = 52): string {
    if (value.length <= max) return value;
    const keep = Math.floor((max - 1) / 2);
    return `${value.slice(0, keep)}…${value.slice(-keep)}`;
  }

  function badgeClass(kind: BackupKind): string {
    if (kind === "manual") {
      return "bg-blue-500/10 text-blue-600 dark:bg-blue-500/15 dark:text-blue-400";
    }
    if (kind === "preRestore") {
      return "bg-amber-500/10 text-amber-600 dark:bg-amber-500/15 dark:text-amber-500";
    }
    return "";
  }

  function statusIcon(state: StatusState) {
    if (state === "ok") return CircleCheckIcon;
    if (state === "running") return LoaderCircleIcon;
    if (state === "stale") return TriangleAlertIcon;
    if (state === "failed") return OctagonXIcon;
    return ClockIcon;
  }

  async function load() {
    loading = true;
    errorMessage = null;
    const result = await commands.getBackupOverview();
    if (result.status === "error") {
      errorMessage = result.error.message;
      loading = false;
      return;
    }
    overview = result.data;
    loading = false;
  }

  async function refresh() {
    const result = await commands.getBackupOverview();
    if (result.status === "error") return;
    overview = result.data;
  }

  async function refreshAll() {
    await refresh();
    await backupAttention.refresh();
  }

  async function save(patch: Partial<BackupSettings>) {
    if (!overview) return;

    const previous = $state.snapshot(overview.settings);
    const next: BackupSettings = { ...previous, ...patch };

    overview.settings = next;
    const result = await commands.setBackupSettings(next);

    if (result.status === "error") {
      overview.settings = previous;
      toast.error("Не вдалося зберегти налаштування", {
        description: result.error.message,
        position: "bottom-left",
      });
      return;
    }

    overview.settings = result.data;
    if ("folder" in patch) {
      await refresh();
    }
    await backupAttention.refresh();
  }

  function onEnabledChange(checked: boolean) {
    void save({ enabled: checked });
  }

  function onIntervalChange(value: string) {
    void save({ intervalHours: Number(value) });
  }

  function onRetentionChange(value: string) {
    void save({ retention: Number(value) });
  }

  async function chooseFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Тека для копій",
    });
    if (typeof selected !== "string") return;
    await save({ folder: selected });
  }

  function resetFolder() {
    void save({ folder: null });
  }

  async function openFolder() {
    const result = await commands.openBackupsDir();
    if (result.status === "error") {
      toast.error("Не вдалося відкрити теку", {
        description: result.error.message,
        position: "bottom-left",
      });
    }
  }

  async function createBackup() {
    if (running || restoring) return;

    creating = true;
    const result = await commands.createBackup();
    creating = false;

    if (result.status === "error") {
      toast.error("Не вдалося створити копію", {
        description: result.error.message,
        position: "bottom-left",
      });
      await refreshAll();
      return;
    }

    toast.success("Резервну копію створено", { position: "bottom-left" });
    await refreshAll();
  }

  function askRestore(backup: BackupInfo) {
    if (backup.isNewer || running || restoring) return;
    restoreTarget = { info: backup, fileName: backup.fileName };
    restoreOpen = true;
  }

  async function restoreFromFile() {
    if (running || restoring) return;

    const selected = await open({
      multiple: false,
      title: "Відновити з файлу",
      filters: [{ name: "Tugboat backup", extensions: ["zip"] }],
    });
    if (typeof selected !== "string") return;

    const result = await commands.inspectBackupFile(selected);
    if (result.status === "error") {
      toast.error("Це не файл резервної копії Tugboat", {
        position: "bottom-left",
      });
      return;
    }

    if (result.data.isNewer) {
      toast.error("Створено новішою версією. Оновіть застосунок.", {
        position: "bottom-left",
      });
      return;
    }

    restoreTarget = { info: result.data, path: selected };
    restoreOpen = true;
  }

  async function confirmRestore() {
    const target = restoreTarget;
    if (!target || running || restoring) return;

    restoring = true;
    const result = target.path
      ? await commands.stageRestoreFile(target.path)
      : await commands.stageRestoreBackup(target.fileName ?? "");

    if (result.status === "error") {
      restoring = false;
      toast.error("Не вдалося відновити дані. Поточні дані не змінено.", {
        description: result.error.message,
        position: "bottom-left",
      });
      return;
    }

    if (dev) {
      // A self-relaunch kills the `tauri dev` session's Vite server, so the
      // new process would open without a frontend. Close instead; the staged
      // restore is applied on the next normal dev launch.
      try {
        await message(
          "Дані підготовлено до відновлення. Застосунок зараз закриється — запустіть його знову, щоб завершити відновлення.",
          { title: "Відновлення даних", kind: "info" },
        );
      } finally {
        try {
          await exit(0);
        } catch {
          await getCurrentWindow().close();
        }
      }
      return;
    }

    toast.success("Застосунок перезапуститься для відновлення", {
      position: "bottom-left",
    });

    try {
      await relaunch();
    } catch (error) {
      toast.error(
        "Не вдалося перезапустити застосунок. Закрийте його та відкрийте знову.",
        {
          description: error instanceof Error ? error.message : String(error),
          position: "bottom-left",
        },
      );
      await getCurrentWindow().close();
    }
  }

  onMount(() => {
    void load();

    const stop = listen("backup-status", () => {
      void refresh();
    });

    return () => {
      void stop.then((unlisten) => unlisten());
    };
  });
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  {#if loading}
    <div class="border-border bg-card mt-4 rounded border p-4">
      <div class="flex items-center gap-3">
        <Skeleton class="size-5 rounded-full" />
        <div class="flex-1">
          <Skeleton class="h-5 w-64 rounded" />
          <Skeleton class="mt-2 h-3.5 w-80 rounded" />
        </div>
        <Skeleton class="h-8 w-32 rounded" />
      </div>
    </div>
  {:else if errorMessage}
    <div class="border-border bg-card mt-4 rounded border p-6 text-center">
      <p class="text-destructive text-sm">{errorMessage}</p>
      <Button class="mt-3" variant="outline" onclick={() => void load()}>
        Спробувати ще раз
      </Button>
    </div>
  {:else if overview && settings}
    {@const StatusIcon = statusIcon(statusState)}
    <div
      class="border-border bg-card mt-4 flex flex-wrap items-center gap-3 rounded border p-4"
    >
      <StatusIcon
        class={cn(
          "size-5 shrink-0",
          statusState === "ok" && "text-emerald-500",
          statusState === "running" && "text-muted-foreground animate-spin",
          statusState === "stale" && "text-amber-500",
          statusState === "failed" && "text-destructive",
          statusState === "empty" && "text-muted-foreground",
        )}
      />

      <div class="min-w-0 flex-1" role="status" aria-live="polite">
        <p class="text-sm font-semibold">{statusHeadline}</p>
        {#if statusDetail}
          <p class="text-muted-foreground mt-0.5 text-xs">{statusDetail}</p>
        {/if}
      </div>

      <div class="flex shrink-0 flex-wrap items-center gap-2">
        {#if statusState === "failed"}
          <Button
            variant="secondary"
            size="sm"
            onclick={() => void chooseFolder()}
          >
            Змінити теку…
          </Button>
          <Button
            size="sm"
            disabled={running || restoring}
            onclick={() => void createBackup()}
          >
            Спробувати ще раз
          </Button>
        {:else if statusState === "running"}
          <Button variant="secondary" size="sm" disabled>Створити копію</Button>
        {:else if statusState === "stale" || statusState === "empty"}
          <Button
            size="sm"
            disabled={running || restoring}
            onclick={() => void createBackup()}
          >
            Створити копію
          </Button>
        {:else}
          <Button
            variant="secondary"
            size="sm"
            disabled={running || restoring}
            onclick={() => void createBackup()}
          >
            Створити копію
          </Button>
        {/if}
      </div>
    </div>

    <p class="my-4 px-4 text-lg font-semibold">Автоматичні копії</p>
    <div class="overflow-hidden border border-border rounded bg-card">
      <Item.Root id="settings-backup-auto">
        <Item.Content>
          <Item.Title>Автоматичні резервні копії</Item.Title>
          <Item.Description>
            Створювати копію під час роботи застосунку
          </Item.Description>
        </Item.Content>
        <Item.Actions>
          <Switch
            checked={settings.enabled ?? true}
            onCheckedChange={onEnabledChange}
            aria-label="Автоматичні резервні копії"
          />
        </Item.Actions>
      </Item.Root>

      <Item.Root class={!settings.enabled ? "opacity-50" : undefined}>
        <Item.Content>
          <Item.Title>Розклад</Item.Title>
          <Item.Description
            >Як часто створювати автоматичну копію</Item.Description
          >
        </Item.Content>
        <Item.Actions>
          <Select.Root
            type="single"
            value={String(settings.intervalHours ?? 24)}
            onValueChange={onIntervalChange}
            disabled={!settings.enabled}
          >
            <Select.Trigger class="w-40">{intervalLabel}</Select.Trigger>
            <Select.Content>
              {#each INTERVAL_OPTIONS as option (option.value)}
                <Select.Item value={String(option.value)} label={option.label}>
                  {option.label}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </Item.Actions>
      </Item.Root>

      <Item.Root class={!settings.enabled ? "opacity-50" : undefined}>
        <Item.Content>
          <Item.Title>Зберігати</Item.Title>
          <Item.Description>
            Кількість копій що зберігаються, старі копії видаляються автоматично
          </Item.Description>
        </Item.Content>
        <Item.Actions>
          <Select.Root
            type="single"
            value={String(settings.retention ?? 14)}
            onValueChange={onRetentionChange}
            disabled={!settings.enabled}
          >
            <Select.Trigger class="w-40"
              >{settings.retention ?? 14}</Select.Trigger
            >
            <Select.Content>
              {#each RETENTION_OPTIONS as option (option.value)}
                <Select.Item value={String(option.value)} label={option.label}>
                  {option.label}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </Item.Actions>
      </Item.Root>
    </div>

    <p class="my-4 px-4 text-lg font-semibold">Розташування</p>
    <div class="overflow-hidden border border-border rounded bg-card">
      <Item.Root id="settings-backup-folder">
        <Item.Content class="min-w-0">
          <Item.Title>Тека для копій</Item.Title>
          <Item.Description>
            Тека куди будуть зберігатися резервні копії
          </Item.Description>

          <div
            class="text-muted-foreground mt-1 flex min-w-0 items-center gap-1.5 text-[13px]"
          >
            <FolderIcon class="size-3.5 shrink-0" />
            <AppTooltip content={folderPath}>
              {#snippet trigger({ props })}
                <span {...props} class="truncate"
                  >{middleTruncate(folderPath)}</span
                >
              {/snippet}
            </AppTooltip>
          </div>
        </Item.Content>

        <Item.Actions class="shrink-0">
          <Button
            variant="secondary"
            size="sm"
            onclick={() => void chooseFolder()}
          >
            Змінити…
          </Button>

          <DropdownMenu.Root>
            <DropdownMenu.Trigger
              class={cn(buttonVariants({ variant: "ghost", size: "icon-sm" }))}
              aria-label="Інші дії з текою"
            >
              <EllipsisIcon class="size-4" />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end" class="w-64">
              <DropdownMenu.Item onSelect={() => void openFolder()}>
                {OPEN_FOLDER_LABEL}
              </DropdownMenu.Item>
              <DropdownMenu.Item
                disabled={!settings.folder}
                onSelect={resetFolder}
              >
                Скинути до теки за замовчуванням
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </Item.Actions>
      </Item.Root>
    </div>

    <div class="flex items-center justify-between gap-3">
      <p class="my-4 px-4 text-lg font-semibold">
        Історія
        <span class="text-muted-foreground ml-2 text-sm font-normal">
          {copiesLabel(backups.length)}
        </span>
      </p>
      <Button
        class="mr-4"
        variant="outline"
        size="sm"
        disabled={running || restoring}
        onclick={() => void restoreFromFile()}
      >
        Відновити з файлу…
      </Button>
    </div>

    <div class="overflow-hidden border border-border rounded bg-card">
      {#if backups.length === 0}
        <p class="text-muted-foreground px-4 py-5 text-sm">Копій ще немає</p>
      {:else}
        <div class="divide-border divide-y">
          {#each historyRows as backup (backup.fileName)}
            <div class="flex items-start gap-3 px-4 py-3">
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <span class="text-sm font-semibold">
                    {formatHistoryDate(backup.createdAt)}
                  </span>
                  <Badge variant="secondary" class={badgeClass(backup.kind)}>
                    {KIND_LABELS[backup.kind]}
                  </Badge>
                </div>
                <p class="text-muted-foreground mt-0.5 text-xs">
                  {formatBytes(backup.sizeBytes)} · версія {backup.appVersion}
                </p>
                {#if backup.isNewer}
                  <p class="text-muted-foreground mt-1 text-xs">
                    Створено новішою версією. Оновіть застосунок.
                  </p>
                {/if}
              </div>

              <Button
                variant="outline"
                size="sm"
                disabled={backup.isNewer || running || restoring}
                onclick={() => askRestore(backup)}
              >
                Відновити
              </Button>
            </div>
          {/each}
        </div>

        {#if !showAll && backups.length > 5}
          <button
            type="button"
            class="border-border text-muted-foreground hover:text-foreground w-full border-t px-4 py-2.5 text-xs font-medium transition-colors"
            onclick={() => (showAll = true)}
          >
            Показати всі ({backups.length})
          </button>
        {/if}
      {/if}
    </div>
  {/if}
</PageLayout>

<AlertDialog.Root bind:open={restoreOpen}>
  <AlertDialog.Content class="sm:max-w-[460px]">
    <AlertDialog.Header>
      <div class="flex items-start gap-3">
        <div
          class="bg-primary/15 text-primary flex size-10 shrink-0 items-center justify-center rounded-xl"
        >
          <FolderIcon class="size-5" />
        </div>
        <div class="flex min-w-0 flex-col gap-1">
          <AlertDialog.Title class="text-base">
            Відновити з копії?
          </AlertDialog.Title>
          <AlertDialog.Description>
            {#if restoreTarget}
              Поточні дані буде замінено даними з копії від {formatHistoryDate(
                restoreTarget.info.createdAt,
              )} (версія {restoreTarget.info.appVersion}).
            {/if}
          </AlertDialog.Description>
        </div>
      </div>
    </AlertDialog.Header>

    <div
      class="border-border bg-muted/40 text-muted-foreground rounded-xl border px-3.5 py-3 text-xs"
    >
      <ul class="list-disc space-y-1.5 pl-4">
        <li>
          Перед відновленням застосунок збереже копію поточних даних, щоб можна
          було повернутися назад.
        </li>
        <li>Застосунок перезапуститься.</li>
      </ul>
    </div>

    <AlertDialog.Footer>
      <AlertDialog.Cancel variant="outline">Скасувати</AlertDialog.Cancel>
      <AlertDialog.Action
        variant="destructive"
        disabled={restoring || running}
        onclick={() => void confirmRestore()}
      >
        {#if restoring}
          <LoaderCircleIcon class="size-4 animate-spin" />
          Відновлюємо…
        {:else}
          Відновити
        {/if}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
