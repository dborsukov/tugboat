<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { resetMode, setMode } from "mode-watcher";
  import { untrack } from "svelte";
  import { fade, fly } from "svelte/transition";

  import CheckIcon from "@lucide/svelte/icons/circle-check";

  import { type AppSettings, type ThemeMode } from "$lib/bindings";

  import { keys } from "$lib/query-keys";

  import * as Card from "$lib/components/ui/card/index";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { debounce } from "./debounce";
  import { createSettingsQuery, createUpdateSettingsMutation } from "./queries";
  import SettingsRow from "./settings-row.svelte";

  const MIN_SPINNER_MS = 300;
  let status = $state<"idle" | "saving" | "saved">("idle");
  let savedTimeout: ReturnType<typeof setTimeout> | null = null;

  const queryClient = useQueryClient();
  const settingsQuery = createSettingsQuery();
  const updateSettingsMutation = createUpdateSettingsMutation(queryClient);

  // local editable copy, seeded once from the first successful load
  let seeded = false;
  let draft = $state<AppSettings | null>(null);
  $effect(() => {
    if (!seeded && settingsQuery.data) {
      seeded = true;
      draft = $state.snapshot(settingsQuery.data);
    }
  });

  function persist() {
    if (!draft) return;

    const startTime = Date.now();
    status = "saving";

    updateSettingsMutation.mutate($state.snapshot(draft), {
      onSuccess: () => {
        const elapsed = Date.now() - startTime;
        const remainingDelay = Math.max(0, MIN_SPINNER_MS - elapsed);
        setTimeout(() => {
          status = "saved";
          if (savedTimeout) clearTimeout(savedTimeout);
          savedTimeout = setTimeout(() => {
            status = "idle";
          }, 3000);
        }, remainingDelay);
      },
      onError: () => {
        // mutation global onError already rolled the cache back, resync draft to it
        status = "idle";
        const server = queryClient.getQueryData<AppSettings>(keys.settings.all);
        if (server) draft = $state.snapshot(server);
      },
    });
  }
  const persistDebounced = debounce(persist, 600);

  const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
    { value: "system", label: "Системна" },
    { value: "light", label: "Світла" },
    { value: "dark", label: "Темна" },
  ];
  const themeLabel = $derived(
    THEME_OPTIONS.find((o) => o.value === draft?.theme)?.label ??
      "Оберіть тему",
  );
  $effect(() => {
    const theme = draft?.theme;
    if (!theme) return;
    untrack(() => {
      theme === "system" ? resetMode() : setMode(theme);
    });
  });

  function setTheme(value: string) {
    if (!draft) return;
    draft.theme = value as ThemeMode;
    persist();
  }

  // cleanup function
  // flush(save) any pending edits
  // if the user navigates away right after typing
  $effect(() => () => persistDebounced.flush());
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full grid justify-end items-center overflow-hidden">
      {#if status === "saving"}
        <span
          in:fly={{ y: 12, duration: 200 }}
          out:fly={{ y: -12, duration: 200 }}
          class="flex gap-2 items-center justify-end col-start-1 row-start-1 text-yellow-600 font-medium"
        >
          <p>Збереження</p>
          <Spinner />
        </span>
      {:else if status === "saved"}
        <span
          in:fly={{ y: 12, duration: 200 }}
          out:fly={{ y: -12, duration: 200 }}
          class="flex gap-2 items-center justify-end col-start-1 row-start-1 text-green-600 font-medium"
        >
          <p>Збережено</p>
          <CheckIcon class="size-4" />
        </span>
      {/if}
    </div>
  {/snippet}
  <div class="p-4 flex flex-col gap-6">
    {#if settingsQuery.isError}
      <FetchErrorDisplay
        message={settingsQuery.error?.message}
        onRetry={settingsQuery.refetch}
        class="mt-20"
      />
    {:else if settingsQuery.isLoading}
      <Skeleton class="h-30 w-full rounded-lg" />
      <Skeleton class="h-30 w-full rounded-lg" />
      <Skeleton class="h-30 w-full rounded-lg" />
    {:else}
      <Card.Root>
        <Card.Header>
          <Card.Title>Оформлення</Card.Title>
        </Card.Header>
        <Card.Content>
          <SettingsRow label="Тема" description="Як має виглядати застосунок.">
            <Select.Root
              type="single"
              value={draft?.theme}
              onValueChange={setTheme}
            >
              <Select.Trigger class="w-30">{themeLabel}</Select.Trigger>
              <Select.Content>
                {#each THEME_OPTIONS as option (option.value)}
                  <Select.Item value={option.value} label={option.label}>
                    {option.label}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </SettingsRow>

          <!-- example free-text field -->
          <!-- <SettingsRow
          >
            <Input
              bind:value={testinput}
              oninput={() => persistDebounced()}
              onblur={() => persistDebounced.flush()}
              onkeydown={(e) => e.key === "Enter" && persistDebounced.flush()}
            />
          </SettingsRow> -->
        </Card.Content>
      </Card.Root>
    {/if}
  </div>
</PageLayout>
