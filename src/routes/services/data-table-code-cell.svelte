<script lang="ts">
  interface Props {
    code: string | null;
  }

  let { code }: Props = $props();
</script>

{#if code}
  <p class="font-mono px-2">{code}</p>
{/if}
