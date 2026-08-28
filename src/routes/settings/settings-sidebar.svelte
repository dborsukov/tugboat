<script lang="ts">
  import { page } from "$app/state";

  import BriefcaseBusinessIcon from "@lucide/svelte/icons/briefcase-business";
  import CarIcon from "@lucide/svelte/icons/car";
  import PaletteIcon from "@lucide/svelte/icons/palette";
  import SearchIcon from "@lucide/svelte/icons/search";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";

  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import SettingsSearchResults from "./settings-search-results.svelte";
  import { settingsSearch } from "./settings-search.svelte";

  const settings_menu = $derived([
    {
      title: "Загальні",
      url: "/settings/general",
      icon: Settings2Icon,
    },
    {
      title: "Зовнішній вигляд",
      url: "/settings/lookandfeel",
      icon: PaletteIcon,
    },
    {
      title: "Замовлення",
      url: "/settings/workorders",
      icon: BriefcaseBusinessIcon,
    },
    {
      title: "Автомобілі",
      url: "/settings/cars",
      icon: CarIcon,
    },
  ]);
</script>

<Sidebar.Root
  collapsible="none"
  variant="sidebar"
  class="border-r border-border"
>
  <Sidebar.Header>
    <InputGroup.Root class="w-full">
      <InputGroup.Input
        placeholder="Пошук..."
        class="w-full"
        bind:value={settingsSearch.query}
      />
      <InputGroup.Addon>
        <SearchIcon class="size-4 shrink-0 opacity-50" />
      </InputGroup.Addon>
    </InputGroup.Root>
  </Sidebar.Header>

  <Sidebar.Content>
    <SettingsSearchResults />
    <Sidebar.Group>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each settings_menu as item (item.title)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                isActive={page.url.pathname.startsWith(item.url)}
              >
                {#snippet child({ props })}
                  <a href={item.url} {...props}>
                    <item.icon />
                    <span>{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
</Sidebar.Root>
