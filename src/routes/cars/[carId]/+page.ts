import { error } from "@sveltejs/kit";

import { appStore } from "$lib/stores/app.svelte";

export const load = async ({ params }) => {
  const car = appStore.cars.find((c) => c.id.toString() === params.carId);

  if (!car) {
    error(404, "Автомобіль не знайдено");
  }

  return { car };
};
