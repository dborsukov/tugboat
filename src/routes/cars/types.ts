import type { Car, CarBrand, Client } from "$lib/bindings";

export type CarListItem = {
  car: Car;
  client: Client;
  brand: CarBrand;
};

export function buildCarListItems(
  cars: Car[],
  clients: Client[],
  carBrands: CarBrand[],
): CarListItem[] {
  return cars.map((car) => {
    const client = clients.find((c) => c.id === car.clientId);
    const brand = carBrands.find((b) => b.id === car.brandId);

    if (!brand || !client) {
      throw new Error(
        `Car ${car.id} references a client/brand that hasn't loaded`,
      );
    }

    return { car, client, brand };
  });
}
