<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Action } from "svelte/action";

  import SheetFrame from "$lib/components/sheet/sheet-frame.svelte";
  import SheetHost from "$lib/components/sheet/sheet-host.svelte";

  let {
    open = $bindable(false),
    formID,
    title,
    description,
    submitting,
    submitLabel,
    onCancel,
    enhanceAction,
    dirty = false,
    bodyClass,
    class: className,
    children,
  }: {
    open: boolean;
    formID: string;
    title: string;
    description: string;
    submitting: boolean;
    submitLabel: string;
    onCancel: () => void;
    enhanceAction: Action<HTMLFormElement>;
    /** True when the form differs from its initial snapshot. */
    dirty?: boolean;
    /** Extra classes for the scrollable form body. */
    bodyClass?: string;
    /** Extra classes for the sheet panel, e.g. a wider `sm:max-w-*` cap. */
    class?: string;
    children: Snippet;
  } = $props();

  const shortcutLabel = $derived(
    submitLabel.toLowerCase().startsWith("зберегти") ? "зберегти" : "створити",
  );
</script>

<SheetHost bind:open autofocus class={className}>
  <SheetFrame
    {formID}
    active={open}
    {title}
    subtitle={description}
    {submitLabel}
    {submitting}
    {dirty}
    {enhanceAction}
    {shortcutLabel}
    onRequestClose={onCancel}
    class={bodyClass}
  >
    {@render children()}
  </SheetFrame>
</SheetHost>
