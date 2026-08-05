<script lang="ts">
  import CopiedIcon from "@lucide/svelte/icons/check";
  import CopyIcon from "@lucide/svelte/icons/copy";

  import { UseClipboard } from "$lib/hooks/use-clipboard.svelte";

  import Button, { type ButtonProps } from "./ui/button/button.svelte";

  const clipboard = new UseClipboard({ delay: 1000 });

  interface Props extends ButtonProps {
    text: string;
    onSuccess?: (() => void) | null;
    onFailure?: (() => void) | null;
  }

  let { text, onSuccess, onFailure, ...props }: Props = $props();
</script>

<Button
  size="icon"
  class="transition-all"
  onclick={() => clipboard.copy(text)}
  {...props}
>
  {#if clipboard.copied}
    <CopiedIcon />
  {:else}
    <CopyIcon />
  {/if}
</Button>
