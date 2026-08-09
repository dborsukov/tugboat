<script lang="ts">
  import type { ComponentProps } from "svelte";
  import { MaskPatterns, PatternText } from "svelte-number-format";

  import DateIcon from "@lucide/svelte/icons/calendar-plus";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import * as Card from "$lib/components/ui/card";

  import type { Client } from "$lib/bindings";
  import { prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import ClientAvatar from "./client-avatar.svelte";

  interface Props extends ComponentProps<typeof Card.Root> {
    client: Client;
    short?: boolean;
  }
  let { client, short, class: className, ...restProps }: Props = $props();
</script>

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
      <Card.Title>
        <p class="text-xl font-semibold text-nowrap truncate text-ellipsis">
          {client.name}
        </p>
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
          <p class="text-base text-nowrap truncate text-ellipsis select-none!">
            <PatternText
              value={client.phone}
              format={MaskPatterns.PHONE_INTERNATIONAL}
            />
          </p>
        </div>
        {#if !short}
          <div
            class="flex items-center gap-2 text-sm text-muted-foreground"
            title="Дата додавання"
          >
            <DateIcon class="size-4 shrink-0" />
            <p class="text-base text-nowrap truncate text-ellipsis">
              {prettifyIsoDateString(client.createdAt, "long")}
            </p>
          </div>
        {/if}
      </Card.Description>
    </div>
  </Card.Header>
</Card.Root>
