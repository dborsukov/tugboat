<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";

  import SetupDot from "$lib/components/setup-dot.svelte";

  import {
    openSetupIssue,
    setupAttention,
  } from "$lib/states/setup-attention.svelte";
  import type { SetupIssue } from "$lib/states/setup-issues";

  const issue = $derived(setupAttention.issue);

  /** Keeps the list visible while the dialog animates closed. */
  let shownIssue = $state<SetupIssue | null>(null);
  $effect(() => {
    if (issue) shownIssue = issue;
  });

  function close() {
    setupAttention.dismiss();
  }

  function openSettings() {
    const target = issue ?? shownIssue;
    if (target) void openSetupIssue(target);
  }
</script>

<Dialog.Root
  open={issue !== null}
  onOpenChange={(open) => {
    if (!open) close();
  }}
>
  <Dialog.Content class="w-[calc(100%-2rem)] sm:max-w-[460px]">
    <Dialog.Header>
      <Dialog.Title>Спочатку заповніть дані бізнесу</Dialog.Title>
      <Dialog.Description>
        Без них у документі не буде даних виконавця.
      </Dialog.Description>
    </Dialog.Header>

    {#if shownIssue}
      <div class="flex flex-col gap-2">
        <p class="text-muted-foreground text-xs font-semibold tracking-wide">
          НЕ ВИСТАЧАЄ
        </p>
        <ul
          class="divide-y divide-border overflow-hidden rounded-xl border border-border"
        >
          {#each shownIssue.fields as field (field.key)}
            <li class="flex items-center gap-2.5 px-3 py-2.5">
              <SetupDot class="size-1.5" />
              <span>{field.label}</span>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <Dialog.Footer>
      <Button variant="ghost" onclick={close}>Скасувати</Button>
      <Button variant="brand" onclick={openSettings}>
        Відкрити налаштування
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
