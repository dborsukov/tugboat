import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params }) => {
  const exists = appStore.work_orders.find(
    (c) => c.id.toString() === params.workorderId,
  );

  if (!exists) {
    error(404, "Замовлення не знайдено");
  }

  return { id: params.workorderId };
};
