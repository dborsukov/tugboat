import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params }) => {
  const exists = appStore.cars.find((c) => c.id.toString() === params.carId);

  if (!exists) {
    error(404, "Автомобіль не знайдено");
  }

  return { id: params.carId };
};
