<script lang="ts">
  import {
    parseDate,
    parseDateTime,
    type DateValue,
  } from "@internationalized/date";

  import CalendarIcon from "@lucide/svelte/icons/calendar";

  import { Button } from "$lib/components/ui/button";
  import { Calendar } from "$lib/components/ui/calendar/index.js";
  import { Input } from "$lib/components/ui/input";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import { cn } from "$lib/utils";

  let {
    value = $bindable(""),
    withTime = false,
    placeholder = "Оберіть дату",
    disabled = false,
    id,
    class: className,
  }: {
    /** `yyyy-MM-dd`, or `yyyy-MM-ddTHH:mm` when `withTime` is set. */
    value?: string;
    withTime?: boolean;
    placeholder?: string;
    disabled?: boolean;
    id?: string;
    class?: string;
  } = $props();

  /** Time applied when a day is picked on a field that has no time yet. */
  const DEFAULT_TIME = "12:00";

  let open = $state(false);

  function pad(n: number): string {
    return String(n).padStart(2, "0");
  }

  function parse(raw: string): DateValue | undefined {
    if (!raw) return undefined;
    try {
      return withTime ? parseDateTime(raw) : parseDate(raw);
    } catch {
      return undefined;
    }
  }

  const selected = $derived(parse(value));
  const time = $derived(value.split("T")[1]?.slice(0, 5) || DEFAULT_TIME);

  const label = $derived.by(() => {
    const date = selected;
    if (!date) return null;
    const day = `${pad(date.day)}.${pad(date.month)}.${date.year}`;
    return withTime ? `${day}, ${time}` : day;
  });

  function selectDate(next: DateValue | undefined) {
    if (!next) {
      value = "";
      return;
    }
    const day = `${next.year}-${pad(next.month)}-${pad(next.day)}`;
    value = withTime ? `${day}T${time}` : day;
    if (!withTime) open = false;
  }

  function selectTime(next: string) {
    const date = selected;
    if (!date || !next) return;
    value = `${date.year}-${pad(date.month)}-${pad(date.day)}T${next}`;
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        {id}
        {disabled}
        variant="outline"
        class={cn(
          "w-full justify-start gap-2 px-2.5 font-normal",
          !label && "text-muted-foreground",
          className,
        )}
      >
        <CalendarIcon class="text-muted-foreground" />
        <span class="min-w-0 flex-1 truncate text-left">
          {label ?? placeholder}
        </span>
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-auto gap-0 p-0" align="start">
    <Calendar
      type="single"
      value={selected}
      onValueChange={selectDate}
      locale="uk-UA"
    />
    {#if withTime}
      <div
        class="flex items-center justify-between gap-3 border-t border-border px-3 py-2.5"
      >
        <span class="text-xs text-muted-foreground">Час</span>
        <Input
          type="time"
          aria-label="Час"
          value={time}
          class="w-28"
          oninput={(event) => selectTime(event.currentTarget.value)}
        />
      </div>
    {/if}
  </Popover.Content>
</Popover.Root>
