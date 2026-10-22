<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import LockIcon from "@lucide/svelte/icons/lock";
  import LockOpenIcon from "@lucide/svelte/icons/lock-open";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";

  import { Button } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import LicensePlate from "$lib/components/license-plate.svelte";
  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import type {
    Client,
    ClientListItem,
    DocumentKind,
    WorkOrderDetailsDto,
  } from "$lib/bindings";
  import { prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import CarBadge from "../../cars/car-badge.svelte";
  import ClientAvatar from "../../clients/client-avatar.svelte";
  import { workerFullName, workerShortName } from "../../workers/labels";
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
    hideNotes = false,
    onGenerate,
    class: className,
  }: {
    workOrderDetails?: WorkOrderDetailsDto;
    isLoading?: boolean;
    isArchived?: boolean;
    isLocked?: boolean;
    clients?: ClientListItem[];
    hideNotes?: boolean;
    onGenerate: (kind: DocumentKind) => void;
    class?: string;
  } = $props();

  const queryClient = useQueryClient();
  const restoreWorkOrderMutation = createRestoreWorkOrderMutation(queryClient);

  const workOrder = $derived(workOrderDetails?.workOrder);
  const client = $derived(workOrderDetails?.client);
  const carDto = $derived(workOrderDetails?.carDto);

  const isTerminal = $derived(
    workOrder?.status === "Paid" ||
      workOrder?.status === "AwaitingPayment" ||
      workOrder?.status === "Completed",
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
   * "Сплачено 10 вересня 2026, 17:12 · Карткою". Gated on `paidAt` rather
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
        ? `${prettifyIsoDateString(entry.value, "table-datetime")}${entry.suffix}`
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

{#snippet fieldBox(
  label: string,
  value: string | number | null | undefined,
  avatarName: string | null = null,
)}
  <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
    <span class="truncate text-sm text-muted-foreground" title={label}
      >{label}</span
    >
    <div class="flex min-w-0 items-center gap-2">
      {#if avatarName}
        <ClientAvatar name={avatarName} class="size-6 shrink-0" />
      {/if}
      <span
        class="truncate font-semibold"
        title={value != null ? String(value) : undefined}>{value ?? "—"}</span
      >
    </div>
  </div>
{/snippet}

{#snippet personContent(
  label: string,
  name: string | null | undefined,
  phone: string | null | undefined,
)}
  <span class="truncate text-sm text-muted-foreground" title={label}
    >{label}</span
  >
  <div class="flex min-w-0 items-start gap-2">
    {#if name}
      <ClientAvatar {name} class="size-6 shrink-0" />
    {/if}
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <span class="truncate font-semibold" title={name ?? undefined}
        >{name ?? "—"}</span
      >
      {#if phone}
        <PhoneDisplay
          value={phone}
          class="block w-full truncate text-sm text-muted-foreground"
        />
      {/if}
    </div>
  </div>
{/snippet}

{#snippet personBox(
  label: string,
  name: string | null | undefined,
  phone: string | null | undefined,
  href: string | null = null,
)}
  {#if href}
    <a
      {href}
      class="group flex min-w-0 items-center gap-3 bg-card px-4 py-3 transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset focus-visible:outline-none"
    >
      <span class="flex min-w-0 flex-1 flex-col gap-1">
        {@render personContent(label, name, phone)}
      </span>
      <ChevronRightIcon
        class="size-4 shrink-0 text-muted-foreground transition-colors group-hover:text-foreground"
        aria-hidden="true"
      />
    </a>
  {:else}
    <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
      {@render personContent(label, name, phone)}
    </div>
  {/if}
{/snippet}

{#snippet notesBox(label: string, value: string)}
  <div class="min-w-0 bg-card px-4 py-3">
    <span class="text-sm text-muted-foreground">{label}</span>
    <p
      class="mt-1 text-sm wrap-break-word italic text-foreground/90 whitespace-pre-wrap"
    >
      {value}
    </p>
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
  <div
    class="overflow-hidden rounded-b border border-t-0 border-border bg-card @container"
  >
    <div
      class="grid grid-cols-4 gap-px border-t border-border bg-border @max-[900px]:grid-cols-2 @max-[560px]:grid-cols-1"
    >
      {#each { length: 4 } as _, i (i)}
        <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
          <Skeleton class="h-4 w-20" />
          <Skeleton class="h-5 w-24" />
        </div>
      {/each}
    </div>
    <div
      class="flex h-14 items-center gap-3 border-t border-border bg-card px-4"
    >
      <Skeleton class="size-8 rounded-full" />
      <Skeleton class="h-4 w-48" />
      <Skeleton class="ml-auto h-4 w-16" />
    </div>
    <div
      class="grid grid-cols-4 gap-px border-t border-border bg-border @max-[900px]:grid-cols-2 @max-[560px]:grid-cols-1"
    >
      {#each { length: 4 } as _, i (i)}
        <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-5 w-32" />
          <Skeleton class="h-4 w-24" />
        </div>
      {/each}
    </div>
    <div
      class="grid grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-px border-t border-border bg-border"
    >
      {#each { length: 3 } as _, i (i)}
        <div class="flex min-w-0 flex-col gap-1.5 bg-card px-4 py-3">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-full" />
          <Skeleton class="h-4 w-2/3" />
        </div>
      {/each}
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
          {prettifyIsoDateString(workOrder.archivedAt ?? "", "table-datetime")}
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
            {:else if workOrder.status === "AwaitingPayment"}
              Замовлення очікує оплати і заблоковано від змін
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
          <h1 class="text-2xl font-mono font-semibold whitespace-nowrap">
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
                      "table-datetime",
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

  <div
    class="overflow-hidden rounded-b border border-t-0 border-border bg-card @container"
  >
    <div
      class="grid grid-cols-4 gap-px border-t border-border bg-border @max-[900px]:grid-cols-2 @max-[560px]:grid-cols-1"
    >
      {@render fieldBox(
        "Виконавець",
        workOrderDetails?.worker
          ? workerShortName(workOrderDetails.worker)
          : null,
        workOrderDetails?.worker
          ? workerFullName(workOrderDetails.worker)
          : null,
      )}
      {@render fieldBox("Пробіг", `${workOrder.odometerIn} км`)}
      {@render fieldBox(
        "Пробіг при видачі",
        workOrder.odometerOut != null ? `${workOrder.odometerOut} км` : null,
      )}
      {@render fieldBox("Тариф", `${workOrder.rate} грн./год.`)}
    </div>

    {#if carDto?.car}
      <a
        href={`/cars/${carDto.car.id}`}
        class="flex w-full items-center gap-3 border-t border-border bg-card px-4 py-3 text-left transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset focus-visible:outline-none"
      >
        <CarBadge
          car={carDto.car}
          carBrand={carDto.brand}
          class="min-w-0 text-base [&>p]:min-w-0 [&>p]:truncate"
        />
        {#if carDto.car.licensePlate}
          <LicensePlate value={carDto.car.licensePlate} />
        {/if}
        <ChevronRightIcon
          class="ml-auto size-4 shrink-0 text-muted-foreground"
        />
      </a>
    {/if}

    <div
      class="grid grid-cols-4 gap-px border-t border-border bg-border @max-[900px]:grid-cols-2 @max-[560px]:grid-cols-1"
    >
      {@render personBox(
        "Власник",
        client?.name,
        phoneOf(ownerContact),
        client ? `/clients/${client.id}` : null,
      )}
      {@render personBox(
        "Замовник",
        customer?.name,
        phoneOf(customer),
        workOrder.customerClientId != null
          ? `/clients/${workOrder.customerClientId}`
          : client
            ? `/clients/${client.id}`
            : null,
      )}
      {@render personBox(
        "Водій / Представник",
        driverName || null,
        workOrder.driverPhone,
        null,
      )}
      {@render personBox(
        "Платник",
        payer?.name,
        phoneOf(payer),
        workOrder.payerClientId != null
          ? `/clients/${workOrder.payerClientId}`
          : client
            ? `/clients/${client.id}`
            : null,
      )}
    </div>

    {#if workOrder?.description || workOrder?.conditionReport || (!hideNotes && workOrder?.notes)}
      <div
        class="grid grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-px border-t border-border bg-border"
      >
        {#if workOrder?.description}
          {@render notesBox("Опис", workOrder.description)}
        {/if}
        {#if workOrder?.conditionReport}
          {@render notesBox("Огляд при прийнятті", workOrder.conditionReport)}
        {/if}
        {#if !hideNotes && workOrder?.notes}
          {@render notesBox("Нотатки", workOrder.notes)}
        {/if}
      </div>
    {/if}
  </div>
{/if}
