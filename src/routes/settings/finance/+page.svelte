<script lang="ts">
  import { goto } from "$app/navigation";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import * as Item from "$lib/components/ui/item/index.js";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { pluralizeUk } from "$lib/tools";

  import { createExpenseCategoriesQuery } from "../../expenses/queries";
  import { SETTINGS_CONTENT_WIDTH_CLASS } from "../settings-context";

  const categoriesQuery = createExpenseCategoriesQuery();

  const categoriesCount = $derived(categoriesQuery.data?.length ?? 0);
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 border border-border rounded bg-card">
    <Item.Root
      id="settings-expense-categories-entry"
      role="button"
      tabindex={0}
      class="hover:cursor-pointer hover:bg-muted/40 focus-visible:bg-muted/40"
      onclick={() => goto("/settings/finance/expense-categories")}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          goto("/settings/finance/expense-categories");
        }
      }}
    >
      <Item.Content>
        <Item.Title>Категорії витрат</Item.Title>
        <Item.Description
          >Створіть категорії та позначте кожну власним кольором</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-3">
          <p class="text-sm text-muted-foreground whitespace-nowrap">
            {categoriesCount}
            {pluralizeUk(categoriesCount, [
              "категорія",
              "категорії",
              "категорій",
            ])}
          </p>
          <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
        </div>
      </Item.Actions>
    </Item.Root>
  </div>
</PageLayout>
