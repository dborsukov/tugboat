<script lang="ts">
  import DateIcon from "@lucide/svelte/icons/calendar-plus";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import type { Client } from "$lib/bindings";

  import { prettifyIsoDateString } from "$lib/utils";

  let { client, short }: { client: Client; short?: boolean } = $props();
</script>

<!-- TODO: add colored avatars based on name -->
<!-- TODO: use shacn Card instead of custom div-->
<!-- TODO: add nav_from prop to add easy reverse navigation -->
<div
  class="rounded-xl border bg-card text-card-foreground shadow-sm p-5 flex flex-col justify-between gap-4 transition-all hover:shadow-lg hover:border-accent-foreground/20 hover:cursor-pointer"
>
  <div class="space-y-1.5">
    <div class="flex items-start gap-2.5">
      <h3
        class="text-xl font-semibold leading-none tracking-tight text-nowrap truncate text-ellipsis"
      >
        {client.name}
      </h3>
    </div>
  </div>

  <div
    class="flex items-center gap-2.5 text-sm text-muted-foreground"
    title="Мобільний номер"
  >
    <PhoneIcon class="size-4 shrink-0" />
    <p class="text-base text-nowrap truncate text-ellipsis">{client.phone}</p>
  </div>
  {#if !short}
    <div
      class="flex items-center gap-2.5 text-sm text-muted-foreground"
      title="Дата додавання"
    >
      <DateIcon class="size-4 shrink-0" />
      <p class="text-base text-nowrap truncate text-ellipsis">
        {prettifyIsoDateString(client.created_at, "long")}
      </p>
    </div>
  {/if}
</div>
