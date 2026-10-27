<script lang="ts">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";
  import type { Action } from "svelte/action";

  import XIcon from "@lucide/svelte/icons/x";

  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";

  import { cn } from "$lib/utils";

  import {
    sheetIconButton,
    sheetPrimaryButton,
    sheetSecondaryButton,
  } from "./field-styles";

  let {
    formID,
    title,
    subtitle,
    submitLabel,
    submitting = false,
    submitDisabled = false,
    dirty = false,
    active = true,
    onRequestClose,
    onBack,
    backLabel,
    stepLabel,
    footerExtra,
    overlay,
    enhanceAction,
    shortcutLabel = "створити",
    class: className,
    children,
  }: {
    formID: string;
    title: string;
    subtitle?: string;
    submitLabel: string;
    submitting?: boolean;
    submitDisabled?: boolean;
    /** When true, leaving asks before throwing the form away. */
    dirty?: boolean;
    /** Only the sheet currently in the foreground listens to Esc / Cmd+Enter. */
    active?: boolean;
    onRequestClose: () => void;
    onBack?: () => void;
    backLabel?: string;
    stepLabel?: string;
    footerExtra?: Snippet;
    /** Optional in-sheet overlay, rendered above the whole panel. */
    overlay?: Snippet;
    enhanceAction: Action<HTMLFormElement>;
    shortcutLabel?: string;
    class?: string;
    children: Snippet;
  } = $props();

  const isMac = $derived(
    typeof navigator !== "undefined" &&
      /Mac|iPhone|iPad|iPod/.test(navigator.platform),
  );

  let formEl = $state<HTMLFormElement | null>(null);
  let discardOpen = $state(false);
  let pendingAction = $state<"close" | "back" | null>(null);
  let keepEditingButton = $state<HTMLButtonElement | null>(null);

  function hasOpenFloatingLayer() {
    return (
      document.querySelector(
        '[data-slot="popover-content"][data-state="open"], [data-slot="select-content"][data-state="open"], [data-slot="dropdown-menu-content"][data-state="open"], [data-slot="alert-dialog-content"], [data-slot="dialog-content"]',
      ) != null
    );
  }

  function requestClose() {
    if (submitting) return;
    if (dirty) {
      pendingAction = "close";
      discardOpen = true;
      return;
    }
    onRequestClose();
  }

  function requestBack() {
    if (!onBack) return;
    if (dirty) {
      pendingAction = "back";
      discardOpen = true;
      return;
    }
    onBack();
  }

  function cancelDiscard() {
    discardOpen = false;
    pendingAction = null;
  }

  function confirmDiscard() {
    const action = pendingAction;
    discardOpen = false;
    pendingAction = null;
    if (action === "back") {
      onBack?.();
      return;
    }
    onRequestClose();
  }

  function hasVisibleEscapeOwner() {
    return [
      ...document.querySelectorAll<HTMLElement>(
        '[data-sheet-escape-owner="true"]',
      ),
    ].some((element) => element.getClientRects().length > 0);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (discardOpen) {
      if (event.key === "Escape") {
        event.preventDefault();
        cancelDiscard();
      }
      return;
    }

    if (!active) return;

    if (event.key === "Escape") {
      if (event.defaultPrevented) return;
      if (hasOpenFloatingLayer()) return;
      if (hasVisibleEscapeOwner()) return;
      event.preventDefault();
      requestClose();
      return;
    }

    const isSubmitChord =
      (event.metaKey || event.ctrlKey) && event.key === "Enter";
    if (!isSubmitChord) return;

    event.preventDefault();
    if (submitting || submitDisabled) return;
    formEl?.requestSubmit();
  }

  $effect(() => {
    const onKeydown = (event: KeyboardEvent) => handleKeydown(event);
    window.addEventListener("keydown", onKeydown, true);
    return () => window.removeEventListener("keydown", onKeydown, true);
  });

  $effect(() => {
    if (!discardOpen) return;
    void tick().then(() => keepEditingButton?.focus());
  });
</script>

<div class="relative flex min-h-0 flex-1 flex-col">
  <header
    class="flex shrink-0 flex-col gap-0 border-b border-border px-6 pt-5 pb-4"
  >
    {#if backLabel}
      <button
        type="button"
        class="-ml-0.5 mb-2 inline-flex w-fit items-center gap-1 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground"
        onclick={requestBack}
      >
        <span aria-hidden="true">←</span>
        {backLabel}
      </button>
    {/if}

    <div class="flex items-start gap-3">
      <div class="min-w-0 flex-1">
        {#if stepLabel}
          <p
            class="mb-1.5 text-[11px] font-semibold tracking-[0.04em] text-brand uppercase"
          >
            {stepLabel}
          </p>
        {/if}
        <h2 class="text-[17px] leading-tight font-semibold tracking-[-0.01em]">
          {title}
        </h2>
        {#if subtitle}
          <p class="mt-1 text-[13px] leading-snug text-muted-foreground">
            {subtitle}
          </p>
        {/if}
      </div>

      <Button
        variant="ghost"
        size="icon-sm"
        class={cn(sheetIconButton, "-mt-1 -mr-1.5")}
        aria-label="Закрити"
        title="Закрити"
        onclick={requestClose}
      >
        <XIcon class="size-4" />
      </Button>
    </div>
  </header>

  <form
    id={formID}
    bind:this={formEl}
    use:enhanceAction
    class={cn(
      "flex min-h-0 flex-1 flex-col gap-4 overflow-x-hidden overflow-y-auto px-6 py-5",
      className,
    )}
  >
    {@render children()}
  </form>

  <footer class="shrink-0 border-t border-border px-6 py-3.5">
    <div class="flex items-center gap-2">
      <span
        class="mr-auto hidden items-center gap-1 text-xs text-muted-foreground/80 sm:inline-flex"
      >
        <span class="tracking-tight">{isMac ? "⌘↵" : "Ctrl+↵"}</span>
        <span>{shortcutLabel}</span>
      </span>

      <Button
        type="button"
        variant="ghost"
        size="sm"
        class="h-9.5 shrink-0 rounded-[8px] px-2.5 text-xs font-medium text-muted-foreground hover:text-foreground"
        onclick={requestClose}
      >
        Скасувати
      </Button>

      {#if footerExtra}
        {@render footerExtra()}
      {/if}

      <Button
        type="submit"
        form={formID}
        variant="brand"
        class={sheetPrimaryButton}
        disabled={submitting || submitDisabled}
        aria-busy={submitting}
      >
        {#if submitting}<Spinner class="size-3.5" />{/if}
        {submitLabel}
      </Button>
    </div>
  </footer>

  {#if overlay}
    {@render overlay()}
  {/if}

  {#if discardOpen}
    <div
      class="absolute inset-0 z-20 flex items-center justify-center bg-background/70 p-6 backdrop-blur-sm"
      role="presentation"
      onclick={(event) => {
        if (event.target === event.currentTarget) cancelDiscard();
      }}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${formID}-discard-title`}
        aria-describedby={`${formID}-discard-description`}
        class="w-full max-w-sm rounded-[12px] border border-border bg-popover p-5 shadow-2xl"
      >
        <h2
          id={`${formID}-discard-title`}
          class="text-[15px] font-semibold tracking-[-0.01em]"
        >
          Відхилити зміни?
        </h2>
        <p
          id={`${formID}-discard-description`}
          class="mt-1.5 text-[13px] leading-relaxed text-muted-foreground"
        >
          Введені дані не збережено. Якщо закрити зараз, їх буде втрачено.
        </p>
        <div class="mt-5 flex items-center justify-end gap-2">
          <Button
            bind:ref={keepEditingButton}
            type="button"
            variant="ghost"
            class={cn(sheetSecondaryButton, "text-muted-foreground")}
            onclick={cancelDiscard}
          >
            Продовжити редагування
          </Button>
          <Button
            type="button"
            variant="destructive"
            class={sheetSecondaryButton}
            onclick={confirmDiscard}
          >
            Відхилити
          </Button>
        </div>
      </div>
    </div>
  {/if}
</div>
