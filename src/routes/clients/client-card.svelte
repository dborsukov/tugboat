<script lang="ts">
  import type { ComponentProps } from "svelte";

  import DateIcon from "@lucide/svelte/icons/calendar-plus";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import type { Client } from "$lib/bindings";

  import { cn, prettifyIsoDateString } from "$lib/utils";

  import * as Card from "$lib/components/ui/card";

  import ClientAvatar from "./client-avatar.svelte";

  interface Props extends ComponentProps<typeof Card.Root> {
    client: Client;
    short?: boolean;
  }
  let { client, short, class: className, ...restProps }: Props = $props();
</script>

<!-- TODO: add nav_from prop to add easy reverse navigation -->
<Card.Root
  class={cn(
    "overflow-hidden transition-all duration-200 rounded hover:shadow-lg hover:border-muted-foreground/30",
    className,
  )}
  {...restProps}
>
  <Card.Header class={cn("flex flex-row items-center gap-4 space-y-0")}>
    <ClientAvatar
      name={client.name}
      class={cn(short ? "size-12" : "size-18")}
    />

    <div class="flex flex-col flex-1 overflow-hidden">
      <Card.Title
        class="text-xl font-semibold leading-none tracking-tight text-nowrap truncate text-ellipsis"
      >
        {client.name}
      </Card.Title>

      <Card.Description
        class="flex flex-col gap-2 mt-2 text-sm text-muted-foreground"
        title="Мобільний номер"
      >
        <div
          class="flex items-center gap-2 text-sm text-muted-foreground"
          title="Дата додавання"
        >
          <PhoneIcon class="size-4 shrink-0" />
          <p class="text-base text-nowrap truncate text-ellipsis">
            {client.phone}
          </p>
        </div>
        {#if !short}
          <div
            class="flex items-center gap-2 text-sm text-muted-foreground"
            title="Дата додавання"
          >
            <DateIcon class="size-4 shrink-0" />
            <p class="text-base text-nowrap truncate text-ellipsis">
              {prettifyIsoDateString(client.created_at, "long")}
            </p>
          </div>
        {/if}
      </Card.Description>
    </div>
  </Card.Header>
</Card.Root>
