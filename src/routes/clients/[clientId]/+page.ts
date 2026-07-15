import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params }) => {
  const exists = appStore.clients.find(
    (c) => c.id.toString() === params.clientId,
  );

  if (!exists) {
    error(404, "Клієнта не знайдено");
  }

  return { id: params.clientId };
};
