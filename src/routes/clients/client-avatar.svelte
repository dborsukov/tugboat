<script lang="ts">
  import * as Avatar from "$lib/components/ui/avatar/index.js";

  import { getAvatarColor, getInitials } from "$lib/avatar.js";
  import { cn } from "$lib/utils.js";

  let {
    name = "",
    src = undefined,
    class: className,
    ...restProps
  }: {
    name?: string;
    src?: string;
    class?: string;
  } = $props();

  const initials = $derived(getInitials(name));
  const colors = $derived(getAvatarColor(name));
</script>

<Avatar.Root class={cn("select-none @container", className)} {...restProps}>
  {#if src}
    <Avatar.Image {src} alt={name} />
  {/if}
  <Avatar.Fallback
    class={cn(
      "font-semibold transition-colors duration-200",
      "text-[45cqw]",
      colors.bg,
      colors.text,
    )}
  >
    {initials}
  </Avatar.Fallback>
</Avatar.Root>
