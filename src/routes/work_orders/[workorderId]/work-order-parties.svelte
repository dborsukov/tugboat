<script lang="ts">
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import { cn } from "$lib/utils";

  import ClientAvatar from "../../clients/client-avatar.svelte";

  type PartyPerson = {
    id: number | null;
    name: string;
    phone: string | null;
  };

  let {
    owner,
    customer = null,
    payer = null,
    driverName = null,
    driverPhone = null,
  }: {
    owner: PartyPerson;
    customer?: PartyPerson | null;
    payer?: PartyPerson | null;
    driverName?: string | null;
    driverPhone?: string | null;
  } = $props();

  function sameParty(a: PartyPerson | null, b: PartyPerson | null): boolean {
    if (a == null || b == null) return false;
    if (a.id != null && b.id != null) return a.id === b.id;
    return a.name === b.name && a.phone === b.phone;
  }

  function partyLabel(roles: string[]): string {
    if (roles.length <= 1) return roles[0] ?? "";
    if (roles.length === 2) {
      return `${roles[0]} і ${roles[1].toLocaleLowerCase("uk")}`;
    }
    return `${roles[0]}, ${roles[1].toLocaleLowerCase("uk")} і ${roles[2].toLocaleLowerCase("uk")}`;
  }

  type PartyCard = {
    key: string;
    label: string;
    person: PartyPerson;
  };

  const customerPerson = $derived(customer ?? owner);
  const payerPerson = $derived(payer ?? owner);
  const customerDiffers = $derived(
    customer != null && !sameParty(customer, owner),
  );
  const payerDiffers = $derived(payer != null && !sameParty(payer, owner));
  const customerIsPayer = $derived(sameParty(customerPerson, payerPerson));

  const cards = $derived.by(() => {
    const result: PartyCard[] = [];
    const ownerRoles = ["Власник"];
    if (!customerDiffers) ownerRoles.push("Замовник");
    if (!payerDiffers) ownerRoles.push("Платник");

    result.push({
      key: "owner",
      label: partyLabel(ownerRoles),
      person: owner,
    });

    if (customerDiffers) {
      const customerRoles = ["Замовник"];
      if (payerDiffers && customerIsPayer) customerRoles.push("Платник");
      result.push({
        key: "customer",
        label: partyLabel(customerRoles),
        person: customerPerson,
      });
    }

    if (payerDiffers && !(customerDiffers && customerIsPayer)) {
      result.push({
        key: "payer",
        label: "Платник",
        person: payerPerson,
      });
    }

    if (driverName || driverPhone) {
      result.push({
        key: "driver",
        label: "Водій / Представник",
        person: {
          id: null,
          name: driverName ?? "",
          phone: driverPhone,
        },
      });
    }

    return result;
  });
</script>

<div
  class={cn(
    "grid gap-px border-t border-border bg-border",
    cards.length === 1
      ? "grid-cols-1"
      : cards.length === 2
        ? "grid-cols-2"
        : cards.length === 3
          ? "grid-cols-3"
          : "grid-cols-4",
    cards.length > 1 && "@max-[900px]:grid-cols-2 @max-[560px]:grid-cols-1",
  )}
>
  {#each cards as card (card.key)}
    {@const href = card.person.id != null ? `/clients/${card.person.id}` : null}
    {#if href}
      <a
        {href}
        class="group flex w-full min-w-0 items-center gap-3 bg-card px-4 py-3 transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset focus-visible:outline-none"
      >
        <span class="flex min-w-0 flex-1 flex-col gap-1">
          {@render personContent(card.label, card.person)}
        </span>
        <ChevronRightIcon
          class="size-4 shrink-0 text-muted-foreground transition-colors group-hover:text-foreground"
          aria-hidden="true"
        />
      </a>
    {:else}
      <div class="flex w-full min-w-0 flex-col gap-1 bg-card px-4 py-3">
        {@render personContent(card.label, card.person)}
      </div>
    {/if}
  {/each}
  {#if cards.length === 3}
    <div
      class="hidden bg-card @max-[900px]:block @max-[560px]:hidden"
      aria-hidden="true"
    ></div>
  {/if}
</div>

{#snippet personContent(label: string, person: PartyPerson)}
  <span class="truncate text-sm text-muted-foreground" title={label}
    >{label}</span
  >
  <div class="flex min-w-0 items-start gap-2">
    {#if person.name}
      <ClientAvatar name={person.name} class="size-6 shrink-0" />
    {/if}
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <span class="truncate font-semibold" title={person.name || undefined}
        >{person.name || "—"}</span
      >
      {#if person.phone}
        <PhoneDisplay
          value={person.phone}
          class="block w-full truncate text-sm text-muted-foreground"
        />
      {/if}
    </div>
  </div>
{/snippet}
