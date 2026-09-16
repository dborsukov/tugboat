<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";
  import type { ComponentProps } from "svelte";

  import {
    cn,
    type WithoutChild,
    type WithoutChildrenOrChild,
  } from "$lib/utils.js";
  import { handleWindowControlsInteractOutside } from "$lib/window-controls.js";

  import AlertDialogOverlay from "./alert-dialog-overlay.svelte";
  import AlertDialogPortal from "./alert-dialog-portal.svelte";

  let {
    ref = $bindable(null),
    class: className,
    size = "default",
    portalProps,
    onInteractOutside,
    ...restProps
  }: WithoutChild<AlertDialogPrimitive.ContentProps> & {
    size?: "default" | "sm";
    portalProps?: WithoutChildrenOrChild<
      ComponentProps<typeof AlertDialogPortal>
    >;
  } = $props();
</script>

<AlertDialogPortal {...portalProps}>
  <AlertDialogOverlay />
  <AlertDialogPrimitive.Content
    bind:ref
    data-slot="alert-dialog-content"
    data-size={size}
    onInteractOutside={(event) => {
      // Working the window controls is not a dismissal: the titlebar stays
      // above the backdrop, so it must stay usable while an alert is open.
      handleWindowControlsInteractOutside(event);
      if (event.defaultPrevented) return;
      onInteractOutside?.(event);
    }}
    class={cn(
      "data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 bg-popover text-popover-foreground ring-foreground/5 dark:ring-foreground/10 gap-6 rounded-[min(var(--radius-4xl),24px)] max-h-[calc(100dvh-var(--overlay-inset,0px)-2rem)] overflow-y-auto p-6 shadow-xl ring-1 duration-100 data-[size=default]:max-w-xs data-[size=sm]:max-w-xs data-[size=default]:sm:max-w-md group/alert-dialog-content fixed left-1/2 z-overlay-content grid w-full -translate-x-1/2 outline-none top-[calc((100dvh+var(--overlay-inset,0px))/2)] -translate-y-1/2",
      className,
    )}
    {...restProps}
  />
</AlertDialogPortal>
