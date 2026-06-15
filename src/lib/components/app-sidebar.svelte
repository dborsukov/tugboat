<script lang="ts">
  import CarsIcon from "@lucide/svelte/icons/car";
  import CustomerIcon from "@lucide/svelte/icons/circle-user";
  import DocumentsIcon from "@lucide/svelte/icons/file";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import { page } from "$app/state";

  import { appStore } from "$lib/stores/app.svelte";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import PopeyeIcon from "$lib/components/icons/popeye.svelte";

  import SidebarFooter from "./ui/sidebar/sidebar-footer.svelte";
  import SidebarMenuBadge from "./ui/sidebar/sidebar-menu-badge.svelte";
  import SidebarMenu from "./ui/sidebar/sidebar-menu.svelte";

  // Menu items.
  const sidebar1 = $derived([
    {
      title: "Клієнти",
      url: "/clients",
      icon: CustomerIcon,
      badge: appStore.clients.length,
    },
    {
      title: "Автомобілі",
      url: "/cars",
      icon: CarsIcon,
      badge: appStore.cars.length,
    },
    {
      title: "Документи",
      url: "/documents",
      icon: DocumentsIcon,
      badge: appStore.documents.length,
    },
  ]);
  const sidebar2 = [
    {
      title: "Сервіси",
      url: "/services",
      icon: WrenchIcon,
    },
  ];
  const footer = [
    {
      title: "Налаштування",
      url: "/settings",
      icon: SettingsIcon,
    },
  ];
</script>

<Sidebar.Root collapsible="icon" variant="sidebar">
  <Sidebar.Header
    class="h-16 border-b justify-center transition-all duration-100 group-data-[collapsible=icon]:h-auto group-data-[collapsible=icon]:py-4"
  >
    <div
      class="flex w-full items-center justify-between flex-row
		group-data-[collapsible=icon]:flex-col
		group-data-[collapsible=icon]:justify-center
		group-data-[collapsible=icon]:gap-4"
    >
      <SidebarMenu>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            isActive={page.url.pathname.endsWith("/")}
          >
            {#snippet child({ props })}
              <a href="/" {...props}>
                <div
                  class="flex aspect-square size-8 items-center justify-center rounded-lg text-sidebar-primary-foreground"
                >
                  <PopeyeIcon class="size-9!" />
                </div>

                <div
                  class="grid flex-1 text-left text-sm leading-tight group-data-[collapsible=icon]:hidden"
                >
                  <span class="truncate font-semibold">Папай CRM </span>
                  <span
                    class="truncate text-xs font-semibold text-muted-foreground"
                    >Версія {appStore.version}</span
                  >
                </div>
              </a>
            {/snippet}
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      </SidebarMenu>

      <div
        class="flex items-center justify-center px-2 group-data-[collapsible=icon]:px-0"
      >
        <Sidebar.Trigger class="h-8 w-8" />
      </div>
    </div>
  </Sidebar.Header>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>База</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each sidebar1 as item (item.title)}
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
              <SidebarMenuBadge>{item.badge}</SidebarMenuBadge>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Управління</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each sidebar2 as item (item.title)}
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
  <SidebarFooter>
    <Sidebar.Menu>
      {#each footer as item (item.title)}
        <Sidebar.MenuItem>
          <Sidebar.MenuButton isActive={page.url.pathname.startsWith(item.url)}>
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
  </SidebarFooter>
</Sidebar.Root>
