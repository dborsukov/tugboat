<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";

  import PlusIcon from "@lucide/svelte/icons/plus";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";

  import PageLayout from "$lib/components/page-layout.svelte";

  import ClientCard from "./client-card.svelte";
  import ClientMutationSheet from "./client-mutation-sheet.svelte";

  let currentPage = $state(1);
  let perPage = $state(15);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);

  let filteredClients = $derived.by(() => {
    const query = searchQuery.trim();
    if (!query) return appStore.clients;

    return appStore.clients
      .map((client) => {
        const nameRank = rankItem(client.name, query);
        const phoneRank = rankItem(client.phone ?? "", query);
        // keep whichever field matched better for this client
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

  let paginatedClients = $derived(
    filteredClients.slice((currentPage - 1) * perPage, currentPage * perPage),
  );

  function handleSearchInput() {
    currentPage = 1;
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <Input
        placeholder="Пошук за ім'ям або телефоном..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
      />
      <Button variant="outline" onclick={() => (isSheetOpen = !isSheetOpen)}>
        Додати <PlusIcon />
      </Button>
      <ClientMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  {#if paginatedClients.length === 0}
    <div class="flex h-40 items-center justify-center text-muted-foreground">
      Клієнтів не знайдено
    </div>
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
