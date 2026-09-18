<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import BanknoteIcon from "@lucide/svelte/icons/banknote";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CircleGaugeIcon from "@lucide/svelte/icons/circle-gauge";
  import CircleUserRoundIcon from "@lucide/svelte/icons/circle-user-round";
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import LockIcon from "@lucide/svelte/icons/lock";
  import LockOpenIcon from "@lucide/svelte/icons/lock-open";
  import MilestoneIcon from "@lucide/svelte/icons/milestone";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";
  import UserRoundIcon from "@lucide/svelte/icons/user-round";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import { Button } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import type {
    Client,
    ClientListItem,
    DocumentFormat,
    DocumentKind,
    WorkOrderDetailsDto,
  } from "$lib/bindings";
  import { prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import { workerShortName } from "../../workers/labels";
  import { paymentMethodInstrumentalLabel } from "../payment-method";
  import { createRestoreWorkOrderMutation } from "../queries";
  import StatusSelector from "../work-order-status-selector.svelte";
  import WorkOrderTotalsPopover from "../work-order-totals-popover.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let {
    workOrderDetails = undefined,
    isLoading = false,
    isArchived = false,
    isLocked = $bindable(false),
    clients = [],
    onGenerate,
    class: className,
  }: {
    workOrderDetails?: WorkOrderDetailsDto;
    isLoading?: boolean;
    isArchived?: boolean;
    isLocked?: boolean;
    clients?: ClientListItem[];
    onGenerate: (kind: DocumentKind, format: DocumentFormat) => void;
    class?: string;
  } = $props();

  const queryClient = useQueryClient();
  const restoreWorkOrderMutation = createRestoreWorkOrderMutation(queryClient);

  const workOrder = $derived(workOrderDetails?.workOrder);
  const client = $derived(workOrderDetails?.client);
  const carDto = $derived(workOrderDetails?.carDto);

  const isTerminal = $derived(
    workOrder?.status === "Paid" || workOrder?.status === "Completed",
  );

  const secondaryLine = $derived(
    [carDto?.car?.licensePlate, carDto?.car?.year].filter(Boolean).join(" · "),
  );

  const clientsById = $derived(new Map((clients ?? []).map((c) => [c.id, c])));

  const ownerContact = $derived(
    workOrderDetails?.clientDetails
      ? {
          ...workOrderDetails.client,
          contactPhone: workOrderDetails.clientDetails.contactPhone,
        }
      : workOrderDetails?.client,
  );

  const customer = $derived(
    workOrder?.customerClientId != null
      ? (clientsById.get(workOrder.customerClientId) ?? null)
      : (ownerContact ?? null),
  );
  const payer = $derived(
    workOrder?.payerClientId != null
      ? (clientsById.get(workOrder.payerClientId) ?? null)
      : (ownerContact ?? null),
  );
  const customerIsOwner = $derived(workOrder?.customerClientId == null);
  const payerIsOwner = $derived(workOrder?.payerClientId == null);

  const driverName = $derived(
    [
      workOrder?.driverSurname,
      workOrder?.driverFirstName,
      workOrder?.driverPatronymic,
    ]
      .filter((part) => part && part.trim() !== "")
      .join(" "),
  );

  const datetimeLabel = $derived(
    workOrder?.paidAt
      ? "Сплачено"
      : workOrder?.completedAt
        ? "Завершено"
        : workOrder?.startedAt
          ? "Почато"
          : "Створено",
  );
  const datetimeIsoString = $derived(
    workOrder?.paidAt ??
      workOrder?.completedAt ??
      workOrder?.startedAt ??
      workOrder?.createdAt ??
      "",
  );

  /**
   * How the order was paid, appended to the date so the header reads
   * "Сплачено 10 вересня 2026 р. о 17:12 · Карткою". Gated on `paidAt` rather
   * than on the status because `paidAt` is what makes the line say "Сплачено";
   * orders paid before the method was recorded simply keep the bare date.
   */
  const paymentMethodSuffix = $derived(
    workOrder?.paidAt && workOrder.paymentMethod
      ? ` · ${paymentMethodInstrumentalLabel(workOrder.paymentMethod)}`
      : "",
  );

  /**
   * Milestones in the order they happen. `suffix` is rendered right after the
   * formatted date; only the paid row carries one (the method that was used),
   * the rest keep an empty one so every row has the same shape.
   */
  const timeline = $derived(
    [
      { label: "Створено", value: workOrder?.createdAt, suffix: "" },
      { label: "Почато", value: workOrder?.startedAt, suffix: "" },
      { label: "Завершено", value: workOrder?.completedAt, suffix: "" },
      {
        label: "Сплачено",
        value: workOrder?.paidAt,
        suffix: paymentMethodSuffix,
      },
    ].map((entry) => ({
      label: entry.label,
      value: entry.value
        ? `${prettifyIsoDateString(entry.value, "datetime")}${entry.suffix}`
        : "—",
    })),
  );

  function phoneOf(
    item: Client | ClientListItem | null | undefined,
  ): string | null {
    if (!item) return null;
    if (item.clientType === "individual") return item.phone;
    return (
      ("contactPhone" in item ? item.contactPhone : null) ?? item.phone ?? null
    );
  }

  /**
   * The identity strip pins to the top of the page while the whole order is
   * scrolled. A sticky element can only travel inside its containing block, so
   * this component deliberately renders the strip and the rest of the header as
   * sibling roots: nested in one wrapper the strip would un-pin as soon as the
   * header itself left the screen. The two boxes are still drawn as one card —
   * squared seam, no double border.
   *
   * The bottom border is always there but only coloured in once the strip is
   * stuck, so the header keeps its current borderless look at rest. It stays a
   * real border in both states because adding and removing it would change the
   * strip's height by a pixel and nudge the page at the exact scroll offset
   * that triggers the swap.
   */
  let stickyBar: HTMLDivElement | null = $state(null);
  let isStuck = $state(false);

  /** Nearest scrollable ancestor: the port the strip sticks to. */
  function scrollPortOf(el: HTMLElement | null): HTMLElement | null {
    for (let p = el?.parentElement; p; p = p.parentElement) {
      if (/(auto|scroll|overlay)/.test(getComputedStyle(p).overflowY)) return p;
    }
    return null;
  }

  $effect(() => {
    const el = stickyBar;
    if (!el) return;

    const port = scrollPortOf(el);
    const update = () => {
      const limit = port ? port.getBoundingClientRect().top : 0;
      isStuck = el.getBoundingClientRect().top <= limit + 0.5;
    };

    update();
    // Scroll events do not bubble, but they do reach window while capturing,
    // so this covers the page's scroll container without holding a reference.
    window.addEventListener("scroll", update, { capture: true, passive: true });
    window.addEventListener("resize", update, { passive: true });
    return () => {
      window.removeEventListener("scroll", update, { capture: true });
      window.removeEventListener("resize", update);
    };
  });

  const stickyBarClass = $derived(
    cn(
      "mt-4 sticky top-0 z-20 rounded-t border border-b bg-card",
      isStuck ? "border-b-border shadow-sm" : "border-b-transparent",
      className,
    ),
  );
</script>

{#snippet personCard(
  label: string,
  Icon: typeof UserRoundIcon,
  name: string | null | undefined,
  options: {
    /** Rendered through PhoneDisplay. Omit for text that isn't a phone
     * number — use `subtitle` instead (e.g. a job position). */
    phone?: string | null;
    subtitle?: string | null;
    /** Muted placeholder */
    hint?: string;
    href?: string;
    loading?: boolean;
    class?: string;
  } = {},
)}
  <div
    class={cn(
      "flex min-h-15 items-start gap-3 rounded border border-border px-3 py-2.5",
      options.class,
    )}
  >
    <span
      class="flex size-9 shrink-0 items-center justify-center rounded border border-border bg-muted/40 text-muted-foreground"
    >
      <Icon class="size-4.5" />
    </span>
    <div class="flex w-full min-w-0 flex-col gap-0.5">
      <!-- title= mirrors every line that truncates, so the full value stays
           readable on hover. -->
      <span class="truncate text-xs text-muted-foreground" title={label}
        >{label}</span
      >
      {#if options.hint}
        <span class="truncate text-sm italic text-muted-foreground"
          >{options.hint}</span
        >
      {:else if options.loading}
        <Skeleton class="my-1 h-4 w-40" />
      {:else if options.href && name}
        <a
          href={options.href}
          class="min-w-0 truncate text-sm font-medium underline-offset-4 hover:underline"
          title={name}
        >
          {name}
        </a>
      {:else}
        <span class="truncate text-sm font-medium" title={name ?? undefined}>
          {name ?? "—"}
        </span>
      {/if}
      {#if !options.hint && !options.loading}
        {#if options.phone}
          <PhoneDisplay
            value={options.phone}
            class="block w-full truncate text-xs text-muted-foreground"
          />
        {/if}
        {#if options.subtitle}
          <span
            class="block w-full truncate text-xs text-muted-foreground"
            title={options.subtitle}
          >
            {options.subtitle}
          </span>
        {/if}
      {/if}
    </div>
  </div>
{/snippet}

{#snippet fieldCard(
  label: string,
  value: string | number | undefined,
  Icon: typeof UserRoundIcon,
)}
  <div
    class="flex min-h-15 items-start gap-3 rounded border border-border px-3 py-2.5"
  >
    <span
      class="flex size-9 shrink-0 items-center justify-center rounded border border-border bg-muted/40 text-muted-foreground"
    >
      <Icon class="size-4.5" />
    </span>
    <div class="flex min-w-0 flex-col gap-0.5">
      <p class="truncate text-xs text-muted-foreground" title={label}>
        {label}
      </p>
      <p class="truncate text-sm font-medium" title={String(value ?? "")}>
        {value}
      </p>
    </div>
  </div>
{/snippet}

{#snippet skeletonHeader()}
  <div class={stickyBarClass} bind:this={stickyBar}>
    <div class="h-15 flex items-center gap-2 border-b border-border px-4">
      <Skeleton class="size-4 rounded-sm" />
      <Skeleton class="h-4 w-64" />
      <Skeleton class="ml-auto h-5 w-24" />
    </div>
    <div class="px-4 py-4">
      <!-- Mirrors the identity row below, wrapping included, so the skeleton
           does not jump when the order arrives. -->
      <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
        <div class="flex min-w-0 flex-col gap-2">
          <Skeleton class="h-7 w-48" />
          <Skeleton class="h-4 w-40" />
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <Skeleton class="h-9 w-32 rounded-md" />
          <Skeleton class="h-9 w-32 rounded-md" />
          <Skeleton class="h-9 w-9 shrink-0 rounded-md" />
        </div>
      </div>
    </div>
  </div>
  <div class="rounded-b border border-t-0 border-border bg-card">
    <div class="flex flex-col gap-4 px-4 pt-0 pb-4">
      <div class="grid grid-cols-[repeat(auto-fit,minmax(13rem,1fr))] gap-1.5">
        {#each { length: 3 } as _, i (i)}
          <div
            class="flex min-h-15 items-start gap-3 rounded border border-border px-4 py-2.5"
          >
            <span class="size-9 shrink-0"></span>
            <div class="flex flex-col gap-1">
              <Skeleton class="h-3.5 w-16" />
              <Skeleton class="h-4 w-24" />
            </div>
          </div>
        {/each}
        <div
          class="flex min-h-15 items-start gap-3 rounded border border-border px-3 py-2.5"
        >
          <Skeleton class="size-9 shrink-0 rounded" />
          <div class="flex flex-col gap-1">
            <Skeleton class="h-3.5 w-16" />
            <Skeleton class="h-4 w-28" />
          </div>
        </div>
      </div>
      <Skeleton class="h-14 rounded border border-border" />
      <div class="grid grid-cols-[repeat(auto-fit,minmax(13rem,1fr))] gap-1.5">
        {#each { length: 4 } as _, i (i)}
          <div
            class="flex min-h-15 items-start gap-3 rounded border border-border px-3 py-2.5"
          >
            <Skeleton class="size-9 shrink-0 rounded" />
            <div class="flex flex-col gap-1">
              <Skeleton class="h-3.5 w-14" />
              <Skeleton class="h-4 w-32" />
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/snippet}

{#if isLoading || !workOrder}
  {@render skeletonHeader()}
{:else}
  <!-- Two roots on purpose: the pinned strip has to be a sibling of the card
       below it, not a child of a shared wrapper, or its containing block would
       end where the header ends and it would scroll away with it. -->
  <!-- Pinned: the lock/archive banner and the identity row stay on screen for
       the whole page, so the status, the total and Розблокувати are always
       reachable from the tables further down. -->
  <div class={stickyBarClass} bind:this={stickyBar}>
    {#if workOrder.type === "Warranty"}
      <div
        class="flex items-center justify-between gap-2 border-b border-border bg-amber-50 px-4 py-2 text-sm dark:bg-amber-500/10"
      >
        <span
          class="flex items-center gap-2 text-amber-800 dark:text-amber-300"
        >
          <ShieldCheckIcon class="size-4 shrink-0" />
          Гарантійний випадок
          {#if workOrder.warrantyOf}
            до {workOrder.warrantyOf}
          {/if}
        </span>
        {#if workOrder.warrantyOf}
          <Button
            variant="link"
            class="p-0"
            onclick={async () =>
              await goto(`/work_orders/${workOrder.warrantyOf}`)}
          >
            Відкрити замовлення
          </Button>
        {/if}
      </div>
    {/if}
    {#if isArchived}
      <div
        class="flex items-center justify-between gap-2 border-b border-border bg-muted/40 px-4 py-2 text-sm"
      >
        <span class="flex items-center gap-2 text-muted-foreground">
          <ArchiveIcon class="size-4" />
          Замовлення архівоване
          {prettifyIsoDateString(workOrder.archivedAt ?? "", "datetime")}
        </span>
        <Button
          variant="link"
          class="p-0"
          onclick={() => restoreWorkOrderMutation.mutate(workOrder.id)}
          disabled={restoreWorkOrderMutation.isPending}
        >
          <ArchiveRestoreIcon />
          Відновити
        </Button>
      </div>
    {:else if isTerminal}
      <div
        class="flex items-center justify-between gap-2 border-b border-border bg-muted/40 px-4 py-2 text-sm"
      >
        <span class="flex items-center gap-2 text-muted-foreground">
          {#if isLocked}
            <LockIcon class="size-4" />
            {#if workOrder.status === "Paid"}
              Замовлення оплачено і заблоковано від змін
            {:else}
              Замовлення завершено і заблоковано від змін
            {/if}
          {:else}
            <LockOpenIcon class="size-4" />
            Замовлення тимчасово розблоковано для редагування
          {/if}
        </span>
        <Button
          variant="link"
          class="p-0"
          onclick={() => (isLocked = !isLocked)}
          aria-label={isLocked
            ? "Розблокувати замовлення"
            : "Заблокувати замовлення"}
          title={isLocked
            ? "Розблокувати замовлення"
            : "Заблокувати замовлення"}
        >
          {isLocked ? "Розблокувати" : "Заблокувати"}
        </Button>
      </div>
    {/if}

    <div class="px-4 py-4">
      <!-- Wrapping flex row rather than a container-query grid: the identity
           block and the actions share a line only while they actually fit.
           A breakpoint cannot get this right — the date line grows with the
           payment method, and the width the query measures is the card, which
           the page can hand a different width than the window shows. Flex
           wrapping needs no threshold and can never push the actions off the
           card. `justify-between` keeps the actions on the right when they fit
           and puts them at the left edge (fully visible) when they wrap. -->
      <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
        <div class="flex min-w-0 flex-col gap-1">
          <h1 class="text-2xl font-semibold tracking-tight whitespace-nowrap">
            {workOrder.id}
          </h1>
          <Popover.Root>
            <Popover.Trigger>
              {#snippet child({ props })}
                <Button
                  variant="ghost"
                  size="sm"
                  {...props}
                  class="px-1.5 -ml-1.5 w-fit gap-1.5 text-sm font-normal text-muted-foreground"
                >
                  <span>{datetimeLabel}</span>
                  <span
                    >{prettifyIsoDateString(
                      datetimeIsoString,
                      "datetime",
                    )}{paymentMethodSuffix}</span
                  >
                  <ChevronDownIcon
                    class="size-4 transition-transform group-aria-expanded/button:rotate-180"
                  />
                </Button>
              {/snippet}
            </Popover.Trigger>
            <Popover.Content class="p-4 w-fit">
              <!-- Two real columns rather than a row per milestone with the
                   value pushed to the right edge: the dates have different
                   lengths, so right-aligning each row left every value starting
                   at its own x. The value column is max-content too, which
                   keeps the dates left-aligned on one vertical line while the
                   popover still shrinks to its widest row. -->
              <div
                class="grid grid-cols-[max-content_max-content] gap-x-6 gap-y-2"
              >
                {#each timeline as entry (entry.label)}
                  <p>{entry.label}:</p>
                  <p>{entry.value}</p>
                {/each}
              </div>
            </Popover.Content>
          </Popover.Root>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <StatusSelector
            {workOrder}
            partItems={workOrderDetails?.partItems ?? []}
            isLocked={isLocked || isArchived}
          />
          {#if workOrderDetails}
            <WorkOrderTotalsPopover {workOrderDetails} />
          {/if}
          <DropdownMenu
            workOrderId={workOrder.id}
            type={workOrder.type}
            odometerIn={workOrder.odometerIn}
            bind:isLocked
            {isArchived}
            {onGenerate}
          />
        </div>
      </div>
    </div>
  </div>

  <div class="rounded-b border border-t-0 border-border bg-card">
    <div class="flex flex-col gap-4 px-4 pt-0 pb-4">
      <div class="grid grid-cols-[repeat(auto-fit,minmax(13rem,1fr))] gap-1.5">
        {@render personCard(
          "Виконавець",
          WrenchIcon,
          workOrderDetails?.worker
            ? workerShortName(workOrderDetails.worker)
            : null,
          {
            hint: workOrderDetails?.worker ? undefined : "не призначено",
          },
        )}
        {@render fieldCard(
          "Пробіг",
          `${workOrder.odometerIn} км`,
          CircleGaugeIcon,
        )}
        {#if workOrder.odometerOut != null}
          {@render fieldCard(
            "Пробіг при видачі",
            `${workOrder.odometerOut} км`,
            MilestoneIcon,
          )}
        {/if}
        {@render fieldCard(
          "Тариф",
          `${workOrder.rate} грн./год.`,
          BanknoteIcon,
        )}
      </div>

      <Button
        variant="ghost"
        class="h-auto w-full justify-start gap-3 rounded border border-border px-4 py-3 font-normal hover:bg-muted/40"
        onclick={async () => {
          await goto(`/cars/${carDto?.car?.id}`);
        }}
      >
        <CarLogoLoader slug={carDto?.brand?.iconSlug ?? ""} class="size-8" />
        <span class="flex min-w-0 flex-col items-start">
          <span class="truncate text-sm font-medium">
            {carDto?.brand?.name}
            {carDto?.car?.model}
          </span>
          {#if secondaryLine}
            <span class="truncate text-xs text-muted-foreground">
              {secondaryLine}
            </span>
          {/if}
        </span>
        <ChevronRightIcon class="ml-auto size-4" />
      </Button>

      <div class="grid grid-cols-[repeat(auto-fit,minmax(13rem,1fr))] gap-1.5">
        {@render personCard("Власник", UserRoundIcon, client?.name, {
          phone: phoneOf(ownerContact),
          href: `/clients/${client?.id}`,
        })}
        {@render personCard("Замовник", FileTextIcon, customer?.name, {
          phone: phoneOf(customer),
          hint: customerIsOwner ? "той самий, що й власник" : undefined,
          href: `/clients/${workOrder.customerClientId}`,
          loading: !customerIsOwner && !customer,
        })}
        {#if driverName || workOrder.driverPhone}
          {@render personCard(
            "Водій / Представник",
            CircleUserRoundIcon,
            driverName,
            { phone: workOrder.driverPhone },
          )}
        {/if}
        {@render personCard("Платник", CreditCardIcon, payer?.name, {
          phone: phoneOf(payer),
          hint: payerIsOwner ? "той самий, що й власник" : undefined,
          href: `/clients/${workOrder.payerClientId}`,
          loading: !payerIsOwner && !payer,
        })}
      </div>
    </div>
  </div>
{/if}
