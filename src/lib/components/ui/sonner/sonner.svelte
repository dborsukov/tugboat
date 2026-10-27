<script lang="ts">
  import { mode } from "mode-watcher";
  import {
    Toaster as Sonner,
    toast,
    type ToasterProps as SonnerProps,
    type ToastOptions,
  } from "svelte-sonner";

  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
  import CircleXIcon from "@lucide/svelte/icons/circle-x";
  import InfoIcon from "@lucide/svelte/icons/info";
  import Loader2Icon from "@lucide/svelte/icons/loader-2";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import XIcon from "@lucide/svelte/icons/x";

  let { ...restProps }: SonnerProps = $props();

  /**
   * Sonner only accepts one Toaster-wide duration, while the app uses a
   * different lifetime per status. Apply the defaults once on the shared toast
   * singleton so call sites keep working untouched; an explicit `duration` on
   * an individual toast still wins.
   */
  const TOAST_DURATIONS = {
    success: 4000,
    info: 4000,
    warning: 6000,
    error: 8000,
    loading: Number.POSITIVE_INFINITY,
  } as const;

  type TimedToastType = keyof typeof TOAST_DURATIONS;

  const DURATION_DEFAULTS_APPLIED = Symbol.for(
    "popeye-crm.toast-duration-defaults",
  );

  if (!Reflect.has(toast, DURATION_DEFAULTS_APPLIED)) {
    type TimedToast = (
      message: string,
      options?: Parameters<typeof toast.success>[1],
    ) => string | number;

    const timedToasts = toast as unknown as Record<TimedToastType, TimedToast>;

    for (const type of Object.keys(TOAST_DURATIONS) as TimedToastType[]) {
      const original = timedToasts[type];
      timedToasts[type] = (message, options) =>
        original(message, {
          duration: TOAST_DURATIONS[type],
          // Errors are the only assertive status in the shared notification
          // surface; the call sites never override this.
          ...(type === "error" ? { important: true } : {}),
          ...options,
        });
    }

    Object.defineProperty(toast, DURATION_DEFAULTS_APPLIED, { value: true });
  }

  const toastOptions: ToastOptions = {
    unstyled: true,
    closeButtonAriaLabel: "Закрити",
    classes: {
      toast: "app-toast",
      title: "app-toast-title",
      description: "app-toast-description",
      icon: "app-toast-icon",
      content: "app-toast-content",
      actionButton: "app-toast-action",
      cancelButton: "app-toast-action",
      closeButton: "app-toast-close",
      loader: "app-toast-loader",
    },
  };
</script>

<!--
  This wrapper owns the visual defaults. Design props are deliberately applied
  after `restProps` so an existing layout-level <Toaster position="..." /> or
  richColors flag cannot bring back the old look.
-->
<Sonner
  {...restProps}
  theme={mode.current}
  position="bottom-right"
  offset={16}
  mobileOffset={16}
  gap={8}
  closeButton
  duration={4000}
  richColors={false}
  {toastOptions}
  class="toaster group"
>
  {#snippet loadingIcon()}
    <Loader2Icon
      class="size-4 animate-spin"
      stroke-width={2}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    />
  {/snippet}
  {#snippet successIcon()}
    <CircleCheckIcon
      class="size-4"
      stroke-width={2}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    />
  {/snippet}
  {#snippet errorIcon()}
    <CircleXIcon
      class="size-4"
      stroke-width={2}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    />
  {/snippet}
  {#snippet infoIcon()}
    <InfoIcon
      class="size-4"
      stroke-width={2}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    />
  {/snippet}
  {#snippet warningIcon()}
    <TriangleAlertIcon
      class="size-4"
      stroke-width={2}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    />
  {/snippet}
  {#snippet closeIcon()}
    <XIcon
      stroke-width={2}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    />
  {/snippet}
</Sonner>

<style>
  /* Semantic toast tokens. Colours are derived from the app palette so an open
     toast follows a live theme switch without a reload. */
  :global([data-sonner-toaster].toaster) {
    --app-toast-surface: var(--popover);
    --app-toast-foreground: var(--popover-foreground);
    --app-toast-border: var(--border);
    --app-toast-shadow: 0 8px 24px
      color-mix(in oklab, var(--foreground) 10%, transparent);
    --app-toast-hover: color-mix(in oklab, var(--muted) 60%, transparent);

    /* Status colours use the same palette as the app's status badges and
       order states (emerald / blue / amber / red). */
    --app-toast-success-tint: var(--color-emerald-100);
    --app-toast-success-icon: var(--color-emerald-700);
    --app-toast-info-tint: var(--color-blue-100);
    --app-toast-info-icon: var(--color-blue-700);
    --app-toast-warning-tint: var(--color-amber-100);
    --app-toast-warning-icon: var(--color-amber-800);
    --app-toast-error-tint: var(--color-red-100);
    --app-toast-error-icon: var(--color-red-700);
    --app-toast-loading-tint: var(--muted);
    --app-toast-loading-icon: var(--muted-foreground);

    /* Sonner writes its 356px default inline, so the supported width
       variable has to win with an important declaration. */
    --width: 380px !important;
    width: var(--width);
    font-family: var(--font-sans);
  }

  :global(html.dark [data-sonner-toaster].toaster) {
    --app-toast-shadow: 0 10px 28px
      color-mix(in oklab, var(--background) 80%, transparent);

    --app-toast-success-tint: color-mix(
      in oklab,
      var(--color-emerald-700) 20%,
      var(--popover)
    );
    --app-toast-success-icon: color-mix(
      in oklab,
      var(--color-emerald-300) 65%,
      var(--color-emerald-500)
    );
    --app-toast-info-tint: color-mix(
      in oklab,
      var(--color-blue-700) 20%,
      var(--popover)
    );
    --app-toast-info-icon: var(--color-blue-300);
    --app-toast-warning-tint: color-mix(
      in oklab,
      var(--color-amber-700) 18%,
      var(--popover)
    );
    --app-toast-warning-icon: color-mix(
      in oklab,
      var(--color-amber-300) 65%,
      var(--color-amber-500)
    );
    --app-toast-error-tint: color-mix(
      in oklab,
      var(--color-red-700) 18%,
      var(--popover)
    );
    --app-toast-error-icon: var(--color-red-400);
  }

  :global([data-sonner-toaster].toaster [data-sonner-toast]) {
    --app-toast-status-tint: var(--app-toast-loading-tint);
    --app-toast-status-icon: var(--app-toast-loading-icon);

    box-sizing: border-box;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    width: var(--width);
    max-width: calc(100vw - 32px);
    padding: 14px;
    border: 1px solid var(--app-toast-border);
    border-radius: 10px;
    background: var(--app-toast-surface);
    color: var(--app-toast-foreground);
    box-shadow: var(--app-toast-shadow);
    font-family: var(--font-sans);
    font-size: 13px;
    line-height: 18px;
  }

  :global(
    [data-sonner-toaster].toaster [data-sonner-toast][data-type="success"]
  ) {
    --app-toast-status-tint: var(--app-toast-success-tint);
    --app-toast-status-icon: var(--app-toast-success-icon);
  }

  :global([data-sonner-toaster].toaster [data-sonner-toast][data-type="info"]) {
    --app-toast-status-tint: var(--app-toast-info-tint);
    --app-toast-status-icon: var(--app-toast-info-icon);
  }

  :global(
    [data-sonner-toaster].toaster [data-sonner-toast][data-type="warning"]
  ) {
    --app-toast-status-tint: var(--app-toast-warning-tint);
    --app-toast-status-icon: var(--app-toast-warning-icon);
  }

  :global(
    [data-sonner-toaster].toaster [data-sonner-toast][data-type="error"]
  ) {
    --app-toast-status-tint: var(--app-toast-error-tint);
    --app-toast-status-icon: var(--app-toast-error-icon);
  }

  :global([data-sonner-toaster].toaster .app-toast-icon) {
    position: relative;
    display: flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--app-toast-status-tint);
    color: var(--app-toast-status-icon);
  }

  :global([data-sonner-toaster].toaster .app-toast-icon svg) {
    width: 16px;
    height: 16px;
    stroke-width: 2px;
  }

  :global([data-sonner-toaster].toaster .app-toast-content) {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding-top: 4px;
  }

  :global([data-sonner-toaster].toaster .app-toast-title) {
    color: var(--app-toast-foreground);
    font-size: 14px;
    font-weight: 600;
    line-height: 20px;
  }

  :global([data-sonner-toaster].toaster .app-toast-description) {
    color: var(--muted-foreground);
    font-size: 13px;
    font-weight: 400;
    line-height: 18px;
  }

  /* The app styles every [data-button] as a coloured text link. This selector
     is intentionally more specific so action and cancel buttons read as
     outline buttons in both themes. */
  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-styled]
      [data-button].app-toast-action
  ) {
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    align-self: flex-start;
    height: 28px;
    margin-left: auto;
    margin-right: 0;
    padding: 0 10px;
    border: 1px solid var(--app-toast-border);
    border-radius: 6px;
    background: transparent;
    color: var(--app-toast-foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
    text-decoration: none;
    white-space: nowrap;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      border-color 120ms ease;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-styled]
      [data-button].app-toast-action:hover
  ) {
    background: var(--app-toast-hover);
    color: var(--app-toast-foreground);
    text-decoration: none;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-styled]
      [data-button].app-toast-action:focus-visible
  ) {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast]
      [data-button].app-toast-action
      + [data-button].app-toast-action
  ) {
    margin-left: 0;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast]
      [data-close-button].app-toast-close
  ) {
    position: absolute;
    top: 8px;
    right: 8px;
    left: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--muted-foreground);
    opacity: 0;
    pointer-events: none;
    transform: none;
    transition:
      opacity 120ms ease,
      background-color 120ms ease,
      color 120ms ease;
    z-index: 1;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast]
      [data-close-button].app-toast-close
      svg
  ) {
    width: 14px;
    height: 14px;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-type="success"]
      [data-close-button],
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-type="info"]
      [data-close-button]
  ) {
    display: none;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-type="warning"]:hover
      [data-close-button],
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-type="warning"]:focus-within
      [data-close-button],
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-type="error"]:hover
      [data-close-button],
    [data-sonner-toaster].toaster
      [data-sonner-toast][data-type="error"]:focus-within
      [data-close-button]
  ) {
    opacity: 1;
    pointer-events: auto;
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast]
      [data-close-button].app-toast-close:hover
  ) {
    background: var(--app-toast-hover);
    color: var(--app-toast-foreground);
  }

  :global(
    [data-sonner-toaster].toaster
      [data-sonner-toast]
      [data-close-button].app-toast-close:focus-visible
  ) {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    :global([data-sonner-toaster].toaster .app-toast-action),
    :global([data-sonner-toaster].toaster .app-toast-close),
    :global([data-sonner-toaster].toaster .app-toast-loader svg) {
      transition: none !important;
      animation: none !important;
    }
  }
</style>
