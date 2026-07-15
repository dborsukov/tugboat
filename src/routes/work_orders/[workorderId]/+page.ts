import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params }) => {
  const work_order = appStore.work_orders.find(
    (c) => c.id.toString() === params.workorderId,
  );

  if (!work_order) {
    error(404, "Замовлення не знайдено");
  }

  return { work_order };
};
