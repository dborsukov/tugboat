<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";

  import type { PageProps } from "./$types";
  import CarPage from "./car-page.svelte";
  import EmptyPage from "./empty-page.svelte";

  let { params }: PageProps = $props();

  const car = $derived(
    appStore.cars.find((c) => c.id.toString() === params.carId),
  );
</script>

{#if car !== undefined}
  <CarPage {car} />
{:else}
  <EmptyPage />
{/if}
