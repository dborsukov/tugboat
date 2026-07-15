import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params }) => {
  const client = appStore.clients.find(
    (c) => c.id.toString() === params.clientId,
  );

  if (!client) {
    error(404, "Клієнта не знайдено");
  }

  return { client };
};
