<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";

  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import ClientCard from "./client-card.svelte";
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

  $effect(() => searchInputEl?.focus());
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full pl-1 flex items-center gap-2">
      <Input
        bind:ref={searchInputEl}
        placeholder="Пошук за ім'ям або телефоном..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        disabled={!clientsQuery.isSuccess || paginatedClients.length === 0}
      />
      <Select.Root
        type="single"
        bind:value={sortBy}
        onValueChange={handleSortChange}
        disabled={!clientsQuery.isSuccess || paginatedClients.length === 0}
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
      <Button variant="outline" onclick={() => (isSheetOpen = !isSheetOpen)}>
        Додати <PlusIcon />
      </Button>
      <ClientMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}

  {#if clientsQuery.isError}
    <FetchErrorDisplay
      message={clientsQuery.error?.message}
      onRetry={clientsQuery.refetch}
      class="mt-20"
    />
  {:else if clientsQuery.isLoading}
    <div
      class="grid gap-4 p-4 grid-cols-[repeat(auto-fit,minmax(200px,405px))] justify-center"
    >
      <Skeleton class="w-full h-35" />
      <Skeleton class="w-full h-35" />
      <Skeleton class="w-full h-35" />
      <Skeleton class="w-full h-35" />
      <Skeleton class="w-full h-35" />
      <Skeleton class="w-full h-35" />
    </div>
  {:else if paginatedClients.length === 0}
    <p class="text-center text-muted-foreground mt-20">Клієнтів не знайдено</p>
  {:else}
    <div
      class="grid gap-4 p-4 grid-cols-[repeat(auto-fit,minmax(200px,405px))] justify-center"
    >
      {#each paginatedClients as client (client.id)}
        <a href="/clients/{client.id}" class="w-full">
          <ClientCard {client} />
        </a>
      {/each}
    </div>
  {/if}

  {#snippet footer()}
    <div class="p-4 flex items-center justify-center">
      <Pagination.Root
        count={filteredClients.length}
        {perPage}
        bind:page={currentPage}
      >
        {#snippet children({ pages, currentPage })}
          <Pagination.Content>
            <Pagination.Item>
              <Pagination.PrevButton />
            </Pagination.Item>

            {#each pages as page (page.key)}
              {#if page.type === "ellipsis"}
                <Pagination.Item>
                  <Pagination.Ellipsis />
                </Pagination.Item>
              {:else}
                <Pagination.Item>
                  <Pagination.Link {page} isActive={currentPage === page.value}>
                    {page.value}
                  </Pagination.Link>
                </Pagination.Item>
              {/if}
            {/each}

            <Pagination.Item>
              <Pagination.NextButton />
            </Pagination.Item>
          </Pagination.Content>
        {/snippet}
      </Pagination.Root>
    </div>
  {/snippet}
</PageLayout>
