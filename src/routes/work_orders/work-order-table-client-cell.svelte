<script lang="ts">
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import UserRoundIcon from "@lucide/svelte/icons/user-round";

  import { Skeleton } from "$lib/components/ui/skeleton";

  import ClientAvatar from "../clients/client-avatar.svelte";
  import type { WorkOrderClientLine } from "./work-order-client-column";

  let {
    mainName,
    others = [],
    loading = false,
  }: {
    mainName: string | null;
    others?: WorkOrderClientLine[];
    loading?: boolean;
  } = $props();
</script>

<div class="flex flex-col gap-1">
  <div class="flex gap-2 items-center">
    <ClientAvatar name={mainName ?? ""} class="size-6" />
    {#if mainName != null}
      <span class="truncate" title={mainName}>{mainName}</span>
    {:else if loading}
      <Skeleton class="h-4 w-32" />
    {:else}
      <span class="text-muted-foreground">—</span>
    {/if}
  </div>
  {#if others.length > 0}
    <div class="flex items-center gap-3 pl-8 text-xs text-muted-foreground">
      {#each others as other (other.role)}
        <span class="flex max-w-56 min-w-0 items-center gap-1">
          {#if other.role === "payer"}
            <CreditCardIcon class="size-3.5 shrink-0" />
          {:else}
            <UserRoundIcon class="size-3.5 shrink-0" />
          {/if}
          <span class="shrink-0 text-foreground/70">{other.label}:</span>
          {#if other.name != null}
            <span class="truncate" title={other.name}>{other.name}</span>
          {:else if loading}
            <Skeleton class="h-3 w-24 shrink-0" />
          {:else}
            <span>—</span>
          {/if}
        </span>
      {/each}
    </div>
  {/if}
</div>
