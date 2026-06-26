import {
  commands,
  type Car,
  type CarBrand,
  type Client,
  type Service,
  type WorkOrder,
} from "$lib/bindings";

function unwrap<T>(
  res: { status: "ok"; data: T } | { status: "error"; error: any },
): T {
  if (res.status === "ok") return res.data;
  const msg =
    res.error && typeof res.error === "object" && "message" in res.error
      ? res.error.message
      : String(res.error);
  throw new Error(msg);
}

class AppStore {
  version: string = $state("0.0.0");
  clients: Client[] = $state([]);
  car_brands: CarBrand[] = $state([]);
  cars: Car[] = $state([]);
  services: Service[] = $state([]);
  work_orders: WorkOrder[] = $state([]);
  loading = $state(false);
  error: string | null = $state(null);

  async loadAll() {
    this.loading = true;
    this.error = null;

    const [clients, brands, cars, services, work_orders, version] =
      await Promise.all([
        commands.fetchClients().then(unwrap),
        commands.fetchCarBrands().then(unwrap),
        commands.fetchCars().then(unwrap),
        commands.fetchServices().then(unwrap),
        commands.fetchWorkOrders().then(unwrap),
        commands.getAppVersion(),
      ]);

    this.clients = clients;
    this.car_brands = brands;
    this.cars = cars;
    this.services = services;
    this.work_orders = work_orders;
    this.version = version;

    this.loading = false;
  }

  createClient(client: Client) {
    this.clients = [...this.clients, client];
  }

  createCar(car: Car) {
    this.cars = [...this.cars, car];
  }

  createService(service: Service) {
    this.services = [...this.services, service];
  }

  createWorkOrder(work_order: WorkOrder) {
    this.work_orders = [...this.work_orders, work_order];
  }

  updateClient(updated: Client) {
    this.clients = this.clients.map((client) =>
      client.id === updated.id ? updated : client,
    );
  }

  updateCar(updated: Car) {
    this.cars = this.cars.map((car) => (car.id === updated.id ? updated : car));
  }

  updateService(updated: Service) {
    this.services = this.services.map((service) =>
      service.id === updated.id ? updated : service,
    );
  }

  updateWorkOrder(updated: WorkOrder) {
    this.work_orders = this.work_orders.map((work_order) =>
      work_order.id === updated.id ? updated : work_order,
    );
  }

  removeClient(id: number) {
    this.clients = this.clients.filter((client) => client.id !== id);
  }

  removeCar(id: number) {
    this.cars = this.cars.filter((car) => car.id !== id);
  }

  // FIXME: actually change is_archive instead of removing
  archiveService(id: number) {
    this.services = this.services.filter((service) => service.id !== id);
  }

  // FIXME: actually change is_archive instead of removing
  archiveWorkOrder(id: string) {
    this.work_orders = this.work_orders.filter(
      (work_order) => work_order.id !== id,
    );
  }
}

export const appStore = new AppStore();
