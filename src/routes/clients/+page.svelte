<script lang="ts">
  import UserPlusIcon from "@lucide/svelte/icons/user-plus";

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

  let filteredClients = $derived(
    appStore.clients.filter((client) => {
      const query = searchQuery.trim().toLowerCase();
      if (!query) return true;

      const matchesName = client.name.toLowerCase().includes(query);
      const matchesPhone = client.phone!.includes(query);

      return matchesName || matchesPhone;
    }),
  );

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
      <!-- TODO: implement fuzzy search with highlight -->
      <Input
        placeholder="Пошук за ім'ям або телефоном..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
      />
      <Button
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        <UserPlusIcon /> Створити
      </Button>
      <ClientMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  {#if paginatedClients.length === 0}
    <div class="flex h-40 items-center justify-center text-muted-foreground">
      Клієнтів не знайдено
    </div>
  {:else}
    <div class="flex flex-wrap gap-4 p-4">
      {#each paginatedClients as client (client.id)}
        <a href="/clients/{client.id}" class="flex-1 min-w-max">
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
