<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";

  import { page } from "$app/state";

  import WorkOrdersIcon from "@lucide/svelte/icons/briefcase-business";
  import CarsIcon from "@lucide/svelte/icons/car";
  import CustomerIcon from "@lucide/svelte/icons/circle-user";
  import HomeIcon from "@lucide/svelte/icons/home";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import PopeyeIcon from "$lib/components/icons/popeye.svelte";

  import { commands } from "$lib/bindings";

  import { carsCountQueryOptions } from "../../routes/cars/queries";
  import { clientsCountQueryOptions } from "../../routes/clients/queries";
  import { servicesCountQueryOptions } from "../../routes/services/queries";
  import { workOrdersCountQueryOptions } from "../../routes/work_orders/queries";
  import SidebarFooter from "./ui/sidebar/sidebar-footer.svelte";
  import SidebarMenuBadge from "./ui/sidebar/sidebar-menu-badge.svelte";

  const combinedQueries = createQueries(() => ({
    queries: [
      clientsCountQueryOptions(),
      carsCountQueryOptions(),
      workOrdersCountQueryOptions(),
      servicesCountQueryOptions(),
      {
        queryKey: ["version"],
        queryFn: () => commands.getAppVersion(),
        staleTime: Infinity,
        gcTime: Infinity,
      },
    ],
    combine: (results) => {
      return {
        clientsCount: results[0],
        carsCount: results[1],
        workOrdersCount: results[2],
        servicesCount: results[3],
        appVersion: results[4],

        isReady: results.every((r) => r.isSuccess),
        isLoading: results.some((r) => r.isPending),
        isError: results.some((r) => r.isError),
        firstError: results.find((r) => r.isError)?.error,

        refetchAll: () => {
          results.forEach((r) => r.refetch());
        },
      };
    },
  }));
  const top_level_menu = $derived([
    {
      title: "Головна",
      url: "/",
      icon: HomeIcon,
      badge: null,
    },
  ]);

  const crm_menu = $derived([
    {
      title: "Клієнти",
      url: "/clients",
      icon: CustomerIcon,
      badge: combinedQueries.isReady ? combinedQueries.clientsCount.data : null,
    },
    {
      title: "Автомобілі",
      url: "/cars",
      icon: CarsIcon,
      badge: combinedQueries.isReady ? combinedQueries.carsCount.data : null,
    },
    {
      title: "Замовлення",
      url: "/work_orders",
      icon: WorkOrdersIcon,
      badge: combinedQueries.isReady
        ? combinedQueries.workOrdersCount.data
        : null,
    },
  ]);

  const management_menu = $derived([
    {
      title: "Сервіси",
      url: "/services",
      icon: WrenchIcon,
      badge: combinedQueries.isReady
        ? combinedQueries.servicesCount.data
        : null,
    },
  ]);
  const footer_menu = [
    {
      title: "Налаштування",
      url: "/settings",
      icon: SettingsIcon,
    },
  ];
</script>

<Sidebar.Root collapsible="icon" variant="sidebar">
  <Sidebar.Header
    class="h-16 overflow-hidden border-b transition-all duration-300 ease-in-out group-data-[collapsible=icon]:h-0 group-data-[collapsible=icon]:border-none group-data-[collapsible=icon]:p-0"
  >
    <div class="flex h-full w-full items-center pl-2">
      <div
        class="flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-lg text-sidebar-primary-foreground transition-all duration-300 ease-in-out group-data-[collapsible=icon]:opacity-0"
      >
        <PopeyeIcon class="size-9! shrink-0" />
      </div>

      <div
        class="flex flex-1 flex-col justify-center overflow-hidden whitespace-nowrap px-2 text-sm font-semibold leading-tight transition-all duration-300 ease-in-out group-data-[collapsible=icon]:opacity-0"
      >
        <span class="truncate">Папай CRM</span>
        <span class="truncate text-xs text-muted-foreground"
          >Версія {combinedQueries.isReady
            ? combinedQueries.appVersion.data
            : "?"}</span
        >
      </div>
    </div>
  </Sidebar.Header>

  <Sidebar.Content class="gap-0">
    <Sidebar.Group class="py-1 group-data-[collapsible=]:mt-2">
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each top_level_menu as item (item.title)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={page.url.pathname === item.url}>
                {#snippet child({ props })}
                  <a href={item.url} {...props}>
                    <item.icon />
                    <span>{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
              {#if item.badge}
                <SidebarMenuBadge>{item.badge}</SidebarMenuBadge>
              {/if}
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>

    <Sidebar.Group class="py-1">
      <Sidebar.GroupLabel
        class="group-data-[collapsible=icon]:pointer-events-none"
        >База</Sidebar.GroupLabel
      >
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each crm_menu as item (item.title)}
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

    <Sidebar.Group class="py-1">
      <Sidebar.GroupLabel
        class="group-data-[collapsible=icon]:pointer-events-none"
        >Управління</Sidebar.GroupLabel
      >
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each management_menu as item (item.title)}
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
  </Sidebar.Content>
  <SidebarFooter>
    <Sidebar.Menu>
      {#each footer_menu as item (item.title)}
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
