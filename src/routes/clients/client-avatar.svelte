<!-- lib/components/ui/UserAvatar.svelte -->
<script lang="ts">
  import { cn } from "$lib/utils.js";

  import * as Avatar from "$lib/components/ui/avatar/index.js";

  const AVATAR_COLORS = [
    {
      bg: "bg-red-100 dark:bg-red-950",
      text: "text-red-700 dark:text-red-300",
    },
    {
      bg: "bg-orange-100 dark:bg-orange-950",
      text: "text-orange-700 dark:text-orange-300",
    },
    {
      bg: "bg-amber-100 dark:bg-amber-950",
      text: "text-amber-700 dark:text-amber-300",
    },
    {
      bg: "bg-emerald-100 dark:bg-emerald-950",
      text: "text-emerald-700 dark:text-emerald-300",
    },
    {
      bg: "bg-teal-100 dark:bg-teal-950",
      text: "text-teal-700 dark:text-teal-300",
    },
    {
      bg: "bg-blue-100 dark:bg-blue-950",
      text: "text-blue-700 dark:text-blue-300",
    },
    {
      bg: "bg-indigo-100 dark:bg-indigo-950",
      text: "text-indigo-700 dark:text-indigo-300",
    },
    {
      bg: "bg-violet-100 dark:bg-violet-950",
      text: "text-violet-700 dark:text-violet-300",
    },
    {
      bg: "bg-purple-100 dark:bg-purple-950",
      text: "text-purple-700 dark:text-purple-300",
    },
    {
      bg: "bg-pink-100 dark:bg-pink-950",
      text: "text-pink-700 dark:text-pink-300",
    },
    {
      bg: "bg-rose-100 dark:bg-rose-950",
      text: "text-rose-700 dark:text-rose-300",
    },
  ];

  let {
    name = "",
    src = undefined,
    class: className,
    ...restProps
  } = $props<{
    name?: string;
    src?: string;
    class?: string;
    [key: string]: any;
  }>();

  const initials = $derived(getInitials(name));
  const colors = $derived(getAvatarColor(name));

  function getInitials(fullName: string): string {
    const cleanName = fullName.trim();
    if (!cleanName) return "?";

    const parts = cleanName.split(/\s+/);

    if (parts.length === 1) {
      return parts[0].slice(0, 2).toUpperCase();
    }

    const first = parts[0][0];
    const last = parts[parts.length - 1][0];
    return (first + last).toUpperCase();
  }

  function getAvatarColor(fullName: string) {
    const cleanName = fullName.trim().toLowerCase();
    if (!cleanName) return { bg: "bg-muted", text: "text-muted-foreground" };

    let hash = 0;
    for (let i = 0; i < cleanName.length; i++) {
      // Simple string hashing (djb2-like algorithm)
      hash = cleanName.charCodeAt(i) + ((hash << 5) - hash);
    }

    const index = Math.abs(hash) % AVATAR_COLORS.length;
    return AVATAR_COLORS[index];
  }
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
