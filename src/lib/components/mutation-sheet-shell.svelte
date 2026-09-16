<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Action } from "svelte/action";

  import { Button } from "$lib/components/ui/button";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  let {
    open = $bindable(false),
    formID,
    title,
    description,
    submitting,
    submitLabel,
    onCancel,
    enhanceAction,
    footer,
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
    footer?: Snippet;
    /** Extra classes for the sheet panel, e.g. a wider `sm:max-w-*` cap. */
    class?: string;
    children: Snippet;
  } = $props();
</script>

<Sheet.Root bind:open>
  <Sheet.Content
    side="right"
    showCloseButton={false}
    class={className}
    onEscapeKeydown={(e) => e.preventDefault()}
    onInteractOutside={(e) => e.preventDefault()}
  >
    <Sheet.Header class="border-b border-border">
      <Sheet.Title>{title}</Sheet.Title>
      <Sheet.Description>{description}</Sheet.Description>
    </Sheet.Header>

    <form
      use:enhanceAction
      id={formID}
      class="flex flex-col p-6 gap-4 overflow-y-auto h-full"
    >
      {@render children()}
    </form>

    <Sheet.Footer class="border-t border-border">
      {#if footer}
        {@render footer()}
      {:else}
        <Button type="submit" disabled={submitting} form={formID}>
          {#if submitting}<Spinner />{/if}
          {submitLabel}
        </Button>
        <Button variant="outline" disabled={submitting} onclick={onCancel}>
          Відмінити та закрити
        </Button>
      {/if}
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
