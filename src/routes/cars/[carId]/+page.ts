import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params, parent }) => {
  await parent();

  const exists = appStore.cars.find((c) => c.id.toString() === params.carId);

  if (!exists) {
    error(404, "Автомобіль не знайдено");
  }

  return { id: params.carId };
};
