<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";
  import { MaskPatterns, PatternText } from "svelte-number-format";

  import { goto } from "$app/navigation";

  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import ClientAvatar from "./client-avatar.svelte";
  import ClientMutationSheet from "./client-mutation-sheet.svelte";
  import { createClientsQuery } from "./queries";

  type SortOption = "date-desc" | "date-asc" | "name-asc" | "name-desc";

  const sortLabels: Record<SortOption, string> = {
    "date-desc": "Спочатку нові",
    "date-asc": "Спочатку старі",
    "name-asc": "За ім'ям (А-Я)",
    "name-desc": "За ім'ям (Я-А)",
  };

  let currentPage = $state(1);
  let perPage = $state(15);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);
  let sortBy = $state<SortOption>("date-desc");
  let searchInputEl = $state<HTMLInputElement | null>(null);

  const clientsQuery = createClientsQuery();

  let filteredClients = $derived.by(() => {
    let clients = clientsQuery.isSuccess ? clientsQuery.data : [];
    const query = searchQuery.trim();
    if (!query) return clients;

    return clients
      .map((client) => {
        const nameRank = rankItem(client.name, query);
        const phoneRank = rankItem(client.phone ?? "", query);
        const bestRank = nameRank.rank > phoneRank.rank ? nameRank : phoneRank;
        return {
          client,
          bestRank,
          passed: nameRank.passed || phoneRank.passed,
        };
      })
      .filter(({ passed }) => passed)
      .sort((a, b) => compareItems(a.bestRank, b.bestRank))
      .map(({ client }) => client);
  });

  let sortedClients = $derived.by(() => {
    const sorted = [...filteredClients];

    switch (sortBy) {
      case "name-asc":
        sorted.sort((a, b) => a.name.localeCompare(b.name, "uk"));
        break;
      case "name-desc":
        sorted.sort((a, b) => b.name.localeCompare(a.name, "uk"));
        break;
      case "date-asc":
        sorted.sort(
          (a, b) =>
            new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
        );
        break;
      case "date-desc":
      default:
        sorted.sort(
          (a, b) =>
            new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
        );
        break;
    }

    return sorted;
  });

  let paginatedClients = $derived(
    sortedClients.slice((currentPage - 1) * perPage, currentPage * perPage),
  );

  function handleSearchInput() {
    currentPage = 1;
  }

  function handleSortChange() {
    currentPage = 1;
  }

  let controlsDisabled = $derived(
    !clientsQuery.isSuccess || (paginatedClients.length === 0 && !searchQuery),
  );

  $effect(() => searchInputEl?.focus());
</script>

<PageLayout contentWidthClass="max-w-5xl" headerWidthClass="max-w-5xl">
  {#snippet header()}
    <div class="h-full flex items-center gap-2">
      <Input
        bind:ref={searchInputEl}
        placeholder="Пошук за ім'ям або телефоном..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        disabled={controlsDisabled}
      />
      <Select.Root
        type="single"
        bind:value={sortBy}
        onValueChange={handleSortChange}
        disabled={controlsDisabled}
      >
        <Select.Trigger class="w-45 shrink-0">
          <SortIcon />
          {sortLabels[sortBy]}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="date-desc">{sortLabels["date-desc"]}</Select.Item>
          <Select.Item value="date-asc">{sortLabels["date-asc"]}</Select.Item>
          <Select.Item value="name-asc">{sortLabels["name-asc"]}</Select.Item>
          <Select.Item value="name-desc">{sortLabels["name-desc"]}</Select.Item>
        </Select.Content>
      </Select.Root>
      <Button
        variant="outline"
        onclick={() => (isSheetOpen = !isSheetOpen)}
        disabled={!clientsQuery.isSuccess}
      >
        Додати <PlusIcon />
      </Button>
      <ClientMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}

  {#if clientsQuery.isError}
    <FetchErrorDisplay
      message={clientsQuery.error?.message}
      onRetry={clientsQuery.refetch}
      class="mt-10"
    />
  {:else if clientsQuery.isLoading}
    <div
      class="mt-4 flex flex-col border border-border rounded divide-y divide-border"
    >
      <Skeleton class="w-full h-14" />
      <Skeleton class="w-full h-14" />
      <Skeleton class="w-full h-14" />
      <Skeleton class="w-full h-14" />
      <Skeleton class="w-full h-14" />
    </div>
  {:else if paginatedClients.length === 0}
    {#if searchQuery}
      <div
        class="mt-4 p-4 flex border border-border rounded bg-card items-center justify-center"
      >
        <p>Пошук не повернув результатів</p>
      </div>
    {:else}
      <div
        class="mt-4 p-4 flex flex-col border border-border rounded bg-card items-center justify-center"
      >
        <p>Клієнтів не знайдено</p>
        <p class="text-muted-foreground">
          Натисніть "Додати +" щоб додати нового клієнта
        </p>
      </div>
    {/if}
  {:else}
    <div
      class="mt-4 flex flex-col border border-border rounded divide-y divide-border bg-card"
    >
      {#each paginatedClients as client (client.id)}
        <Button
          variant="ghost"
          class="w-full h-auto justify-start rounded-none px-5 py-3 first:pt-4 last:pb-4"
          onclick={async () => await goto(`/clients/${client.id}`)}
        >
          <div class="flex w-full items-center gap-3.5">
            <ClientAvatar name={client?.name} class="size-9 shrink-0" />
            <p class="flex-1 min-w-0 truncate text-left text-base font-medium">
              {client?.name}
            </p>
            <PatternText
              value={client?.phone}
              format={MaskPatterns.PHONE_INTERNATIONAL}
              class="shrink-0 text-sm text-muted-foreground select-none!"
            />
            <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
          </div>
        </Button>
      {/each}
    </div>

    <div class="p-4 flex items-center justify-center">
      <Pagination.Root
        count={filteredClients.length}
        {perPage}
        bind:page={currentPage}
      >
        {#snippet children({ pages, currentPage })}
          <Pagination.Content class="gap-5">
            <Pagination.Item>
              <Pagination.PrevButton
                class="border-none shadow-none text-muted-foreground hover:text-foreground"
              />
            </Pagination.Item>

            {#each pages as page (page.key)}
              {#if page.type === "ellipsis"}
                <Pagination.Item>
                  <Pagination.Ellipsis />
                </Pagination.Item>
              {:else}
                <Pagination.Item>
                  <Pagination.Link
                    {page}
                    isActive={currentPage === page.value}
                    class="border-none shadow-none {currentPage === page.value
                      ? 'font-semibold text-foreground'
                      : 'text-muted-foreground'}"
                  >
                    {page.value}
                  </Pagination.Link>
                </Pagination.Item>
              {/if}
            {/each}

            <Pagination.Item>
              <Pagination.NextButton
                class="border-none shadow-none text-muted-foreground hover:text-foreground"
              />
            </Pagination.Item>
          </Pagination.Content>
        {/snippet}
      </Pagination.Root>
    </div>
  {/if}
</PageLayout>
