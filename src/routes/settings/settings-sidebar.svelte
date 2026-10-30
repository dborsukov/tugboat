<script lang="ts">
  import { page } from "$app/state";

  import BriefcaseBusinessIcon from "@lucide/svelte/icons/briefcase-business";
  import CarIcon from "@lucide/svelte/icons/car";
  import LandmarkIcon from "@lucide/svelte/icons/landmark";
  import PaletteIcon from "@lucide/svelte/icons/palette";
  import SearchIcon from "@lucide/svelte/icons/search";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";

  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import SetupDot from "$lib/components/setup-dot.svelte";

  import { getSetupIssues } from "$lib/states/setup-issues";

  import SettingsSearchResults from "./settings-search-results.svelte";
  import { settingsSearch } from "./settings-search.svelte";

  const setupIssues = $derived(getSetupIssues());

  function hasIssue(url: string): boolean {
    return setupIssues.some((issue) => issue.page.startsWith(url));
  }

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
    {
      title: "Фінанси",
      url: "/settings/finance",
      icon: LandmarkIcon,
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
            {@const active = page.url.pathname.startsWith(item.url)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={active}>
                {#snippet child({ props })}
                  <a href={item.url} {...props}>
                    <item.icon />
                    <span class="min-w-0 flex-1 truncate">{item.title}</span>
                    {#if hasIssue(item.url)}
                      <SetupDot
                        class="ring-2 {active
                          ? 'ring-sidebar-accent'
                          : 'ring-sidebar group-hover/menu-button:ring-sidebar-accent'}"
                      />
                    {/if}
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
