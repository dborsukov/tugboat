<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";

  import { page } from "$app/state";

  import WorkOrdersIcon from "@lucide/svelte/icons/briefcase-business";
  import CarsIcon from "@lucide/svelte/icons/car";
  import PlusIcon from "@lucide/svelte/icons/circle-plus";
  import CustomerIcon from "@lucide/svelte/icons/circle-user";
  import ClipboardListIcon from "@lucide/svelte/icons/clipboard-list";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import HomeIcon from "@lucide/svelte/icons/home";
  import ListXIcon from "@lucide/svelte/icons/list-x";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import UsersIcon from "@lucide/svelte/icons/users";
  import WalletIcon from "@lucide/svelte/icons/wallet";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import SetupDot from "$lib/components/setup-dot.svelte";

  import type {
    ExpenseSummary,
    MarginSummary,
    RecentItem,
    RevenueSummary,
    SidebarMetric,
    WorkOrderStatus,
  } from "$lib/bindings";
  import {
    clearRecentItems,
    createRecentItemsQuery,
    recentItemsVersion,
  } from "$lib/recent-items";
  import { quickOrder } from "$lib/states/quick-order.svelte";
  import { settingsState } from "$lib/states/settings.svelte";
  import { getSetupIssues } from "$lib/states/setup-issues";

  import { carsCountQueryOptions } from "../../routes/cars/queries";
  import { clientsCountQueryOptions } from "../../routes/clients/queries";
  import { formatCurrencyNumber, toNumber } from "../../routes/home/format";
  import { currentMonthFinancialSummaryQueryOptions } from "../../routes/home/queries";
  import { workOrdersCountQueryOptions } from "../../routes/work_orders/queries";
  import SidebarFooter from "./ui/sidebar/sidebar-footer.svelte";
  import SidebarMenuBadge from "./ui/sidebar/sidebar-menu-badge.svelte";

  const combinedQueries = createQueries(() => ({
    queries: [
      clientsCountQueryOptions(),
      carsCountQueryOptions(),
      workOrdersCountQueryOptions(),
    ],
    combine: (results) => {
      return {
        clientsCount: results[0],
        carsCount: results[1],
        workOrdersCount: results[2],

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

  const financialQueries = createQueries(() => {
    const summary = currentMonthFinancialSummaryQueryOptions();

    return {
      queries: summary.queries,
      combine: (results) => {
        const income = results[0].data as MarginSummary | undefined;
        const expenses = results[1].data as ExpenseSummary | undefined;
        const revenue = results[2].data as RevenueSummary | undefined;

        return {
          month: summary.month,
          ready: results.every((result) => result.isSuccess),
          revenue: toNumber(revenue?.month.total),
          grossIncome: toNumber(income?.income),
          financialResult: toNumber(income?.income) - toNumber(expenses?.total),
        };
      },
    };
  });

  const recentQuery = createRecentItemsQuery();
  const recentItems = $derived(recentQuery.data ?? []);

  $effect(() => {
    $recentItemsVersion;
    void recentQuery.refetch();
  });

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

  const management_menu = [
    {
      title: "Працівники",
      url: "/workers",
      icon: UsersIcon,
    },
    {
      title: "Сервіси",
      url: "/services",
      icon: WrenchIcon,
    },
  ];

  const finance_menu = [
    {
      title: "Витрати",
      url: "/expenses",
      icon: WalletIcon,
    },
    {
      title: "Звіти",
      url: "/reports",
      icon: FileSpreadsheetIcon,
    },
  ];

  const footer_menu = [
    {
      title: "Налаштування",
      url: "/settings",
      icon: SettingsIcon,
    },
  ];

  const recentIcons = {
    client: CustomerIcon,
    car: CarsIcon,
    work_order: ClipboardListIcon,
  };

  const recentStatusDots: Record<WorkOrderStatus, string> = {
    Received: "bg-slate-400",
    InDiagnostics: "bg-sky-500",
    AwaitingApproval: "bg-amber-500",
    AwaitingParts: "bg-amber-500",
    Working: "bg-blue-500",
    ReadyForPickup: "bg-blue-500",
    AwaitingPayment: "bg-orange-500",
    Completed: "bg-teal-500",
    Paid: "bg-green-500",
    Canceled: "bg-rose-500",
  };

  function capitalize(value: string): string {
    return value.charAt(0).toUpperCase() + value.slice(1);
  }

  function recentHref(item: RecentItem): string {
    switch (item.kind) {
      case "client":
        return `/clients/${item.id}`;
      case "car":
        return `/cars/${item.id}`;
      case "work_order":
        return `/work_orders/${item.id}`;
    }
  }

  const isMac = $derived(
    typeof navigator !== "undefined" &&
      /Mac|iPhone|iPad|iPod/.test(navigator.platform),
  );

  function handleNewOrder() {
    quickOrder.openFromCurrentRoute();
  }

  const homeMonth = $derived(
    financialQueries.month ? capitalize(financialQueries.month) : "",
  );
  const sidebarMetric = $derived(
    (settingsState.settings?.sidebarMetric ??
      "financial_result") as SidebarMetric,
  );
  const homeResult = $derived.by(() => {
    if (!financialQueries.ready) return null;

    switch (sidebarMetric) {
      case "revenue":
        return financialQueries.revenue;
      case "gross_income":
        return financialQueries.grossIncome;
      case "financial_result":
        return financialQueries.financialResult;
    }
  });
  const showRecentItems = $derived(
    settingsState.settings?.showRecentItems ?? true,
  );
  const setupIssues = $derived(getSetupIssues());
  const hasSetupIssues = $derived(setupIssues.length > 0);
  const isHomeActive = $derived(page.url.pathname === "/home");

  const RECENT_BOTTOM_GAP = 12;
  let recentVisibleCount = $state(10);
  const visibleRecentItems = $derived(recentItems.slice(0, recentVisibleCount));

  function fitRecentRows(node: HTMLElement, itemCount: number) {
    let count = itemCount;

    const measure = () => {
      if (!node.isConnected || count === 0) return;
      const available = node.clientHeight - RECENT_BOTTOM_GAP;
      if (available <= 0) return;

      const row = node.querySelector("li");
      const menu = node.querySelector("ul");
      const rowHeight =
        row instanceof HTMLElement ? row.getBoundingClientRect().height : 32;
      const menuGap =
        menu instanceof HTMLElement
          ? Number.parseFloat(getComputedStyle(menu).rowGap) || 2
          : 2;
      if (!Number.isFinite(rowHeight) || rowHeight <= 0) return;

      recentVisibleCount = Math.min(
        count,
        Math.max(0, Math.floor((available + menuGap) / (rowHeight + menuGap))),
      );
    };

    // The action runs before the browser has laid out the freshly mounted
    // viewport, so measure once now and once after layout.
    measure();
    const frame = requestAnimationFrame(measure);
    const observer = new ResizeObserver(measure);
    observer.observe(node);

    return {
      update(nextCount: number) {
        count = nextCount;
        measure();
      },
      destroy() {
        cancelAnimationFrame(frame);
        observer.disconnect();
      },
    };
  }
</script>

<Sidebar.Root collapsible="icon" variant="sidebar">
  <Sidebar.Content class="gap-1">
    <Sidebar.Group>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton
              isActive={isHomeActive}
              size="lg"
              class="h-12 w-full min-w-0 bg-transparent dark:bg-slate-600/10 border border-sidebar-border px-1.5 py-2 hover:bg-sidebar-accent dark:hover:bg-sidebar-accent/40 data-active:bg-sidebar-accent dark:data-active:bg-sidebar-accent/60 group-data-[collapsible=icon]:border-0"
            >
              {#snippet child({ props })}
                <a href="/home" {...props}>
                  <span
                    class="flex size-8.5 shrink-0 items-center justify-center rounded-xl transition-colors group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:bg-transparent {isHomeActive
                      ? 'bg-white/55 text-sky-600 dark:bg-sky-100/5 dark:text-blue-300'
                      : 'bg-slate-200/50 text-sky-600/80 dark:bg-slate-700/40 dark:text-blue-300/80'}"
                  >
                    <HomeIcon />
                  </span>
                  <span class="flex min-w-0 flex-col">
                    <span>Головна</span>
                    <span
                      class="flex min-w-0 items-center gap-1.5 text-xs font-normal text-muted-foreground group-data-[collapsible=icon]:hidden"
                    >
                      <span class="truncate">{homeMonth}</span>
                      <span aria-hidden="true">·</span>
                      <span class="truncate">
                        {homeResult != null
                          ? `${formatCurrencyNumber(homeResult)} грн.`
                          : "—"}
                      </span>
                    </span>
                  </span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>

          <Sidebar.MenuItem class="mt-2.5">
            <Button
              variant="outline"
              size="lg"
              class="w-full justify-start gap-2 rounded-xl border-sidebar-border bg-transparent px-3 font-normal shadow-none hover:bg-slate-100 dark:bg-transparent dark:hover:bg-slate-800/60 group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:shrink-0 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:p-0"
              aria-keyshortcuts={isMac ? "Meta+N" : "Control+N"}
              onclick={handleNewOrder}
            >
              <PlusIcon />
              <span class="group-data-[collapsible=icon]:hidden"
                >Швидке замовлення</span
              >
              <kbd
                class="ml-auto shrink-0 font-sans text-[10px] font-medium tracking-wide text-muted-foreground group-data-[collapsible=icon]:hidden"
              >
                {isMac ? "⌘N" : "Ctrl+N"}
              </kbd>
            </Button>
          </Sidebar.MenuItem>
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
              {#if item.badge != null}
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
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>

    <Sidebar.Group>
      <Sidebar.GroupLabel
        class="group-data-[collapsible=icon]:pointer-events-none"
        >Фінанси</Sidebar.GroupLabel
      >
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each finance_menu as item (item.title)}
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

    {#if showRecentItems}
      <Sidebar.Group
        class="min-h-0 flex-1 origin-left overflow-hidden transition-[opacity,transform] duration-200 ease-out group-data-[collapsible=icon]:pointer-events-none group-data-[collapsible=icon]:-translate-x-2 group-data-[collapsible=icon]:opacity-0"
      >
        <div
          class="flex h-8 shrink-0 items-center justify-between pl-3 pr-1 text-xs font-medium text-sidebar-foreground/70"
        >
          <span>Нещодавні</span>
          {#if recentItems.length > 0}
            <button
              type="button"
              class="group/clear inline-flex h-6 min-w-6 max-w-6 cursor-pointer items-center justify-end gap-0 overflow-hidden rounded-md px-1 text-sidebar-foreground/45 transition-[max-width,gap,padding,background-color,color] duration-200 ease-out hover:max-w-28 hover:gap-1.5 hover:bg-sidebar-accent/60 hover:px-2 hover:text-sidebar-foreground"
              title="Очистити"
              aria-label="Очистити"
              onclick={() => void clearRecentItems()}
            >
              <span
                class="max-w-0 overflow-hidden text-xs whitespace-nowrap opacity-0 transition-[max-width,opacity] duration-200 ease-out group-hover/clear:max-w-20 group-hover/clear:opacity-100"
              >
                Очистити
              </span>
              <ListXIcon class="size-3.5 shrink-0" />
            </button>
          {/if}
        </div>
        <div
          use:fitRecentRows={recentItems.length}
          class="flex min-h-0 flex-1 flex-col overflow-hidden"
        >
          {#if recentQuery.isPending}
            <p class="px-3 py-2 text-xs text-muted-foreground">Завантаження…</p>
          {:else if recentItems.length === 0}
            <p class="px-3 py-2 text-xs text-muted-foreground text-pretty">
              Тут з’являться останні клієнти, автомобілі та замовлення.
            </p>
          {:else}
            <Sidebar.Menu>
              {#each visibleRecentItems as item (item.kind + ":" + item.id)}
                {@const Icon = recentIcons[item.kind]}
                {@const dot = item.status
                  ? recentStatusDots[item.status]
                  : "bg-muted-foreground"}
                {@const current = page.url.pathname === recentHref(item)}
                <Sidebar.MenuItem>
                  <Sidebar.MenuButton
                    class={current
                      ? "text-sm text-sidebar-foreground font-semibold"
                      : "text-sm text-sidebar-foreground"}
                  >
                    {#snippet child({ props })}
                      <a
                        href={recentHref(item)}
                        {...props}
                        aria-current={current ? "page" : undefined}
                      >
                        <Icon />
                        {#if item.subtitle}
                          <span class="shrink-0">{item.label}</span>
                          <span
                            class="min-w-0 truncate text-sidebar-foreground/55"
                          >
                            {item.subtitle}
                          </span>
                        {:else}
                          <span class="min-w-0 truncate">{item.label}</span>
                        {/if}
                        {#if item.status}
                          <span
                            class="ml-auto size-2 shrink-0 rounded-full {dot}"
                          ></span>
                        {/if}
                      </a>
                    {/snippet}
                  </Sidebar.MenuButton>
                </Sidebar.MenuItem>
              {/each}
            </Sidebar.Menu>
          {/if}
        </div>
      </Sidebar.Group>
    {/if}
  </Sidebar.Content>
  <SidebarFooter>
    <Sidebar.Menu>
      {#each footer_menu as item (item.title)}
        {@const active = page.url.pathname.startsWith(item.url)}
        <Sidebar.MenuItem>
          <Sidebar.MenuButton isActive={active}>
            {#snippet child({ props })}
              <a href={item.url} {...props}>
                <item.icon />
                <span>{item.title}</span>
                {#if item.url === "/settings" && hasSetupIssues}
                  <SetupDot
                    class="ml-auto ring-2 {active
                      ? 'ring-sidebar-accent'
                      : 'ring-sidebar group-hover/menu-button:ring-sidebar-accent'} group-data-[collapsible=icon]:hidden"
                  />
                {/if}
              </a>
            {/snippet}
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      {/each}
    </Sidebar.Menu>
  </SidebarFooter>
</Sidebar.Root>
