<script lang="ts">
  import type { Table } from "@tanstack/table-core";

  import UserPlusIcon from "@lucide/svelte/icons/circle-plus";

  import type { Service } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";
  import DataTable from "./data-table.svelte";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");
  let isSheetOpen: boolean = $state(false);
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <Input
        placeholder="Пошук за назвою або ціною..."
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        <UserPlusIcon /> Додати
      </Button>
      <ServiceMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  <DataTable data={appStore.services} {columns} bind:searchTerm={inputValue} />
</PageLayout>
