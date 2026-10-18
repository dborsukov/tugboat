<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import SettingsIcon from "@lucide/svelte/icons/settings";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Switch } from "$lib/components/ui/switch";
  import * as Tooltip from "$lib/components/ui/tooltip";

  import { settingsState } from "$lib/states/settings.svelte";

  import {
    resolveLayout,
    toSavedLayout,
    withHidden,
    withSetting,
    type LayoutEntry,
    type ModuleSettings,
  } from "../dashboard-layout";
  import { VAT_DUE_ID } from "../module-ids";
  import { createDashboardLayoutMutation } from "../queries";

  const queryClient = useQueryClient();
  const dashboardLayout = createDashboardLayoutMutation(queryClient);

  let open = $state(false);

  /** Every module the registry ships, switched-off ones included */
  const entries = $derived(
    resolveLayout(settingsState.settings?.dashboardLayout),
  );

  /** The id of the module whose settings dialog is open, if any */
  let settingsFor = $state<string | null>(null);

  /** Module the dialog is open on, as it stands now. */
  const settingsEntry = $derived(
    entries.find((entry) => entry.id === settingsFor) ?? null,
  );

  /** Saves a whole arrangement rather than a patch */
  function commit(next: readonly LayoutEntry[]) {
    if (next === entries) return;
    dashboardLayout.mutate(
      toSavedLayout(next, settingsState.settings?.dashboardLayout),
    );
  }

  function setSetting(
    entry: LayoutEntry | null,
    setting: keyof ModuleSettings,
    value: boolean | number,
  ) {
    if (!entry) return;

    commit(withSetting(entries, entry.id, setting, value));
  }

  function switchID(id: string) {
    return `dashboard-module-${id}`;
  }
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="outline"
        size="icon-lg"
        aria-label="Модулі головної сторінки"
        onclick={() => (open = true)}
      >
        <SlidersHorizontalIcon />
      </Button>
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content side="bottom">Модулі головної сторінки</Tooltip.Content>
</Tooltip.Root>

<Sheet.Root bind:open>
  <Sheet.Content side="right">
    <Sheet.Header class="border-b border-border">
      <Sheet.Title>Модулі головної сторінки</Sheet.Title>
      <Sheet.Description>
        Оберіть які модулі показувати на головній сторінці, та змініть їх
        налаштування.
      </Sheet.Description>
    </Sheet.Header>

    <div class="flex flex-col gap-1 overflow-y-auto p-6">
      {#each entries.filter((entry) => entry.module.id !== VAT_DUE_ID || settingsState.isVatPayer) as entry (entry.id)}
        <div class="flex items-center gap-1 px-2 py-2">
          <label
            for={switchID(entry.id)}
            class="flex-1 cursor-pointer text-sm {entry.hidden
              ? 'text-muted-foreground'
              : ''}"
          >
            {entry.module.title}
          </label>

          {#if entry.module.settings?.length}
            <Tooltip.Root>
              <Tooltip.Trigger>
                {#snippet child({ props })}
                  <Button
                    {...props}
                    variant="ghost"
                    size="icon-sm"
                    aria-label="Налаштування: {entry.module.title}"
                    onclick={() => (settingsFor = entry.id)}
                  >
                    <SettingsIcon class="text-muted-foreground" />
                  </Button>
                {/snippet}
              </Tooltip.Trigger>
              <Tooltip.Content side="bottom">Налаштування</Tooltip.Content>
            </Tooltip.Root>
          {/if}

          <Switch
            id={switchID(entry.id)}
            checked={!entry.hidden}
            onCheckedChange={(checked) =>
              commit(withHidden(entries, entry.id, !checked))}
          />
        </div>
      {/each}
    </div>
  </Sheet.Content>
</Sheet.Root>

<Dialog.Root
  open={settingsEntry !== null}
  onOpenChange={(next) => (settingsFor = next ? settingsFor : null)}
>
  <Dialog.Content class="gap-4">
    <Dialog.Header>
      <Dialog.Title>Налаштування: {settingsEntry?.module.title}</Dialog.Title>
    </Dialog.Header>

    <div class="flex flex-col gap-4">
      {#each settingsEntry?.module.settings ?? [] as setting (setting.id)}
        <div class="flex items-start justify-between gap-4">
          <div class="flex flex-col gap-1">
            <label for={`setting-${setting.id}`} class="cursor-pointer text-sm">
              {setting.label}
            </label>
            <p class="text-muted-foreground text-xs">
              {setting.description}
            </p>
          </div>

          {#if setting.kind === "number"}
            {@const storedNumber = settingsEntry?.settings[setting.id]}
            {@const chosen =
              typeof storedNumber === "number" ? storedNumber : setting.default}
            <Select.Root
              type="single"
              value={String(chosen)}
              onValueChange={(next) =>
                setSetting(settingsEntry, setting.id, Number(next))}
            >
              <Select.Trigger
                id={`setting-${setting.id}`}
                class="w-20 shrink-0"
                aria-label={setting.label}
              >
                {chosen}
              </Select.Trigger>
              <Select.Content>
                {#each Array.from({ length: setting.range.max - setting.range.min + 1 }, (_, index) => setting.range.min + index) as option (option)}
                  <Select.Item value={String(option)} label={String(option)}>
                    {option}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          {:else}
            {@const storedSwitch = settingsEntry?.settings[setting.id]}
            <Switch
              id={`setting-${setting.id}`}
              class="mt-0.5 shrink-0"
              checked={typeof storedSwitch === "boolean" ? storedSwitch : false}
              onCheckedChange={(checked) =>
                setSetting(settingsEntry, setting.id, checked)}
            />
          {/if}
        </div>
      {/each}
    </div>
  </Dialog.Content>
</Dialog.Root>
