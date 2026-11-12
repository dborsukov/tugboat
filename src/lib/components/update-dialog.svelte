<script lang="ts">
  import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
  import ArrowUpRightIcon from "@lucide/svelte/icons/arrow-up-right";
  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronUpIcon from "@lucide/svelte/icons/chevron-up";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import InfoIcon from "@lucide/svelte/icons/info";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import XIcon from "@lucide/svelte/icons/x";

  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";

  import { brandName } from "$lib/brand";
  import {
    parseChangelog,
    plainChangelog,
    type ChangelogKind,
    type ChangelogSection,
  } from "$lib/changelog";
  import { updateState } from "$lib/states/update.svelte";

  const MAX_VISIBLE_ITEMS = 3;

  const KIND_STYLE: Record<ChangelogKind, { card: string; heading: string }> = {
    new: { card: "bg-emerald-500/10", heading: "text-emerald-400" },
    improved: { card: "bg-sky-500/10", heading: "text-sky-400" },
    fixed: { card: "bg-amber-500/10", heading: "text-amber-400" },
  };

  const sections = $derived(parseChangelog(updateState.body));
  const paragraphs = $derived(plainChangelog(updateState.body));

  let expanded = $state<Record<string, boolean>>({});
  let listEl = $state<HTMLDivElement | null>(null);
  let cardEl = $state<HTMLDivElement | null>(null);
  let showFade = $state(false);
  let lastVersion = "";

  const title = $derived.by(() => {
    switch (updateState.phase) {
      case "available":
        return "Доступна нова версія";
      case "downloading":
      case "installing":
        return `Оновлення до версії ${updateState.version}`;
      case "installed":
        return "Оновлення встановлено";
      case "error":
        return "Не вдалося оновити";
      default:
        return "Оновлення";
    }
  });

  const description = $derived.by(() => {
    switch (updateState.phase) {
      case "available":
        return `${brandName()} ${updateState.version} готовий до встановлення`;
      case "downloading":
        return "Не закривайте застосунок, поки триває завантаження";
      case "installing":
        return "Встановлюємо оновлення…";
      case "installed":
        return "Залишився лише перезапуск";
      case "error":
        return "Завантаження або встановлення перервалося";
      default:
        return "";
    }
  });

  const percent = $derived(updateState.percent);
  const sizeLabel = $derived.by(() => {
    const total = updateState.total;
    if (total === null) return formatBytes(updateState.downloaded);
    return `${formatBytes(updateState.downloaded)} з ${formatBytes(total)}`;
  });
  const etaLabel = $derived(formatEta(updateState.etaSeconds));
  const dateLabel = $derived(formatDate(updateState.date));

  $effect(() => {
    if (updateState.version !== lastVersion) {
      lastVersion = updateState.version;
      expanded = {};
    }
  });

  // The list is the only elastic region: measure how much room the fixed rows leave and cap it there.
  $effect(() => {
    void sections;
    void expanded;
    void listEl;
    void cardEl;
    requestAnimationFrame(fitList);
  });

  $effect(() => {
    const onResize = () => requestAnimationFrame(fitList);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  function visibleItems(section: ChangelogSection): string[] {
    if (expanded[section.title]) return section.items;
    return section.items.slice(0, MAX_VISIBLE_ITEMS);
  }

  function hiddenCount(section: ChangelogSection): number {
    return Math.max(0, section.items.length - MAX_VISIBLE_ITEMS);
  }

  function toggleSection(title: string) {
    expanded[title] = !expanded[title];
  }

  function fitList() {
    const card = cardEl;
    const list = listEl;
    if (!card || !list) return;

    list.style.maxHeight = "none";
    const computedLimit = Number.parseFloat(
      window.getComputedStyle(card).maxHeight,
    );
    const cardLimit = Number.isFinite(computedLimit)
      ? computedLimit
      : Math.min(640, window.innerHeight - 40);
    const naturalCard = card.scrollHeight;
    if (naturalCard > cardLimit) {
      const naturalList = list.scrollHeight;
      const overflow = naturalCard - cardLimit;
      list.style.maxHeight = `${Math.max(140, naturalList - overflow - 2)}px`;
    }

    measureFade();
  }

  function measureFade() {
    const element = listEl;
    if (!element) {
      showFade = false;
      return;
    }
    const remaining =
      element.scrollHeight - element.scrollTop - element.clientHeight;
    showFade = remaining > 8;
  }

  function pluralChanges(count: number): string {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return "зміна";
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
      return "зміни";
    }
    return "змін";
  }

  function formatBytes(bytes: number): string {
    if (!Number.isFinite(bytes) || bytes <= 0) return "0 МБ";
    const megabytes = bytes / (1024 * 1024);
    if (megabytes < 1) return `${Math.max(1, Math.round(bytes / 1024))} КБ`;
    return `${megabytes.toFixed(1)} МБ`;
  }

  function formatDate(value: string | null): string | null {
    if (!value) return null;
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return null;
    return new Intl.DateTimeFormat("uk-UA", {
      day: "numeric",
      month: "long",
      year: "numeric",
    })
      .format(date)
      .replace(/\s*р\.$/, "");
  }

  function formatEta(seconds: number | null): string | null {
    if (seconds === null || !Number.isFinite(seconds)) return null;
    if (seconds < 1) return "менше секунди";
    if (seconds < 60) return `≈ ${Math.round(seconds)} с`;
    return `≈ ${Math.ceil(seconds / 60)} хв`;
  }
</script>

<Dialog.Root
  open={updateState.open}
  onOpenChange={(open) => {
    if (!open) updateState.dismiss();
  }}
>
  <Dialog.Content
    bind:ref={cardEl}
    onOpenAutoFocus={(event) => {
      event.preventDefault();
      requestAnimationFrame(() => {
        if (!cardEl) return;
        cardEl.tabIndex = -1;
        cardEl.focus();
      });
    }}
    class="flex max-h-[min(640px,calc(100dvh-var(--overlay-inset,0px)-2rem))] flex-col gap-0 overflow-hidden overflow-y-hidden p-0 sm:max-w-[500px]"
    showCloseButton={false}
  >
    {#if updateState.phase !== "downloading" && updateState.phase !== "installing"}
      <button
        type="button"
        class="text-muted-foreground hover:bg-secondary hover:text-foreground absolute top-4 right-4 inline-flex size-7 items-center justify-center rounded-md transition-colors"
        aria-label="Закрити"
        onclick={() => updateState.dismiss()}
      >
        <XIcon class="size-4" />
      </button>
    {/if}

    <div class="flex shrink-0 items-start gap-3 p-6 pb-4 pr-12">
      <div
        class="bg-primary/15 text-primary flex size-10 shrink-0 items-center justify-center rounded-xl"
      >
        {#if updateState.phase === "installed"}
          <CheckIcon class="size-5" />
        {:else}
          <DownloadIcon class="size-5" />
        {/if}
      </div>
      <div class="flex min-w-0 flex-col gap-0.5">
        <Dialog.Title class="text-base">{title}</Dialog.Title>
        <Dialog.Description>{description}</Dialog.Description>
      </div>
    </div>

    {#if updateState.phase === "available"}
      <div class="flex flex-col gap-4 px-6 pb-5">
        <div
          class="border-border bg-muted/40 flex shrink-0 items-center justify-between gap-2 rounded-xl border px-3 py-2.5 text-xs"
        >
          <div class="flex items-center gap-1.5">
            <span class="text-muted-foreground">
              {updateState.currentVersion}
            </span>
            <ArrowRightIcon class="text-muted-foreground size-3.5" />
            <span class="font-medium">{updateState.version}</span>
            <Badge variant="success">Нова</Badge>
          </div>
          {#if dateLabel}
            <span class="text-muted-foreground">{dateLabel}</span>
          {/if}
        </div>

        {#if sections.length > 0}
          <div class="flex flex-col gap-2">
            <p
              class="text-muted-foreground text-xs font-semibold tracking-wide"
            >
              ЩО НОВОГО
            </p>
            <div class="relative">
              <div
                bind:this={listEl}
                onscroll={measureFade}
                class="space-y-2 overflow-y-auto overscroll-contain pr-1"
              >
                {#each sections as section (section.title)}
                  <div
                    class="rounded-xl px-3.5 py-3 {KIND_STYLE[section.kind]
                      .card}"
                  >
                    <p
                      class="text-xs font-semibold {KIND_STYLE[section.kind]
                        .heading}"
                    >
                      {section.title}
                    </p>
                    <ul
                      class="mt-1.5 list-disc space-y-1 pl-4 text-sm marker:text-foreground/40"
                    >
                      {#each visibleItems(section) as item, index (index)}
                        <li class="leading-snug">{item}</li>
                      {/each}
                    </ul>
                    {#if hiddenCount(section) > 0}
                      <button
                        type="button"
                        class="mt-2 inline-flex items-center gap-1 text-xs font-medium {KIND_STYLE[
                          section.kind
                        ].heading} hover:underline"
                        onclick={() => toggleSection(section.title)}
                      >
                        {#if expanded[section.title]}
                          Згорнути
                          <ChevronUpIcon class="size-3" />
                        {:else}
                          Ще {hiddenCount(section)}
                          {pluralChanges(hiddenCount(section))}
                          <ChevronDownIcon class="size-3" />
                        {/if}
                      </button>
                    {/if}
                  </div>
                {/each}
              </div>
              {#if showFade}
                <div
                  class="from-popover pointer-events-none absolute inset-x-0 bottom-0 h-10 bg-gradient-to-t to-transparent"
                ></div>
              {/if}
            </div>
          </div>
        {:else if paragraphs.length > 0}
          <div
            class="text-muted-foreground flex max-h-[min(360px,42dvh)] flex-col gap-1 overflow-y-auto overscroll-contain pr-1 text-sm"
          >
            {#each paragraphs as paragraph (paragraph)}
              <p>{paragraph}</p>
            {/each}
          </div>
        {/if}

        <button
          type="button"
          class="text-primary inline-flex w-fit shrink-0 items-center gap-1 text-xs hover:underline"
          onclick={() => updateState.openReleaseNotes()}
        >
          Усі зміни у версії {updateState.version}
          <ArrowUpRightIcon class="size-3" />
        </button>

        <div
          class="text-muted-foreground flex shrink-0 items-start gap-2 text-xs"
        >
          <InfoIcon class="mt-0.5 size-3.5 shrink-0" />
          <span>
            Після встановлення застосунок перезапуститься. Ваші дані
            збережуться.
          </span>
        </div>
      </div>
    {:else if updateState.phase === "downloading"}
      <div class="flex flex-col gap-2 px-6 pb-5">
        <div class="flex items-center justify-between text-sm">
          <span>Завантаження…</span>
          {#if percent !== null}
            <span class="text-muted-foreground tabular-nums">{percent} %</span>
          {/if}
        </div>
        <div
          class="bg-muted h-1.5 w-full overflow-hidden rounded-full"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={percent ?? undefined}
          aria-label="Завантаження оновлення"
        >
          <div
            class="bg-primary h-full rounded-full transition-[width] duration-300"
            style:width={`${percent ?? 0}%`}
          ></div>
        </div>
        <div
          class="text-muted-foreground flex items-center justify-between text-xs tabular-nums"
        >
          <span>{sizeLabel}</span>
          <span>{etaLabel ?? ""}</span>
        </div>
        <div class="text-muted-foreground flex items-start gap-2 pt-1 text-xs">
          <InfoIcon class="mt-0.5 size-3.5 shrink-0" />
          <span>
            Після завантаження застосунок перезапуститься автоматично.
          </span>
        </div>
      </div>
    {:else if updateState.phase === "installing"}
      <div class="flex items-center gap-2 px-6 pb-5 text-sm">
        <RotateCwIcon class="size-4 animate-spin" />
        <span>Встановлення оновлення…</span>
      </div>
    {:else if updateState.phase === "installed"}
      <div class="flex flex-col gap-2 px-6 pb-5 text-sm">
        <p>Версію {updateState.version} встановлено.</p>
        <p class="text-muted-foreground">
          Перезапуск через
          <span class="text-foreground font-medium tabular-nums">
            {updateState.restartIn} с
          </span>
          , щоб застосувати зміни.
        </p>
      </div>
    {:else if updateState.phase === "error"}
      <p class="text-destructive px-6 pb-5 text-sm">
        {updateState.error ?? "Невідома помилка"}
      </p>
    {/if}

    <div
      class="border-border flex shrink-0 items-center gap-2 border-t px-6 py-4"
    >
      {#if updateState.phase === "available"}
        <Button
          variant="ghost"
          class="text-muted-foreground mr-auto hover:bg-transparent"
          onclick={() => updateState.skipVersion()}
        >
          Пропустити цю версію
        </Button>
        <Button variant="outline" onclick={() => updateState.remindLater()}>
          Нагадати пізніше
        </Button>
        <Button onclick={() => void updateState.downloadNow()}>
          <DownloadIcon class="size-4" />
          Оновити
        </Button>
      {:else if updateState.phase === "downloading"}
        <Button
          variant="outline"
          class="ml-auto"
          onclick={() => updateState.cancel()}
        >
          Скасувати
        </Button>
      {:else if updateState.phase === "installed"}
        <Button
          variant="ghost"
          class="mr-auto"
          onclick={() => updateState.dismiss()}
        >
          Пізніше
        </Button>
        <Button onclick={() => updateState.restartNow()}>
          <RotateCwIcon class="size-4" />
          Перезапустити зараз
        </Button>
      {:else if updateState.phase === "error"}
        <Button
          variant="ghost"
          class="mr-auto"
          onclick={() => updateState.close()}
        >
          Закрити
        </Button>
        <Button onclick={() => void updateState.retry()}>
          Спробувати ще раз
        </Button>
      {:else if updateState.phase === "installing"}
        <span class="text-muted-foreground ml-auto text-xs">
          Не закривайте застосунок
        </span>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
