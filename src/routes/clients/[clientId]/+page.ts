import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params, parent }) => {
  await parent();

  const exists = appStore.clients.find(
    (c) => c.id.toString() === params.clientId,
  );

  if (!exists) {
    error(404, "Клієнта не знайдено");
  }

  return { id: params.clientId };
};
