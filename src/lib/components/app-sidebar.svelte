<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";

  import { page } from "$app/state";

  import WorkOrdersIcon from "@lucide/svelte/icons/briefcase-business";
  import CarsIcon from "@lucide/svelte/icons/car";
  import CustomerIcon from "@lucide/svelte/icons/circle-user";
  import HomeIcon from "@lucide/svelte/icons/home";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import UsersIcon from "@lucide/svelte/icons/users";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import { carsCountQueryOptions } from "../../routes/cars/queries";
  import { clientsCountQueryOptions } from "../../routes/clients/queries";
  import { servicesCountQueryOptions } from "../../routes/services/queries";
  import { workOrdersCountQueryOptions } from "../../routes/work_orders/queries";
  import { workersCountQueryOptions } from "../../routes/workers/queries";
  import SidebarFooter from "./ui/sidebar/sidebar-footer.svelte";
  import SidebarMenuBadge from "./ui/sidebar/sidebar-menu-badge.svelte";

  const combinedQueries = createQueries(() => ({
    queries: [
      clientsCountQueryOptions(),
      carsCountQueryOptions(),
      workOrdersCountQueryOptions(),
      servicesCountQueryOptions(),
      workersCountQueryOptions(),
    ],
    combine: (results) => {
      return {
        clientsCount: results[0],
        carsCount: results[1],
        workOrdersCount: results[2],
        servicesCount: results[3],
        workersCount: results[4],

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
      title: "Працівники",
      url: "/workers",
      icon: UsersIcon,
      badge: combinedQueries.isReady ? combinedQueries.workersCount.data : null,
    },
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
  <Sidebar.Content class="gap-1">
    <Sidebar.Group>
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

    <Sidebar.Group>
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

    <Sidebar.Group>
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
