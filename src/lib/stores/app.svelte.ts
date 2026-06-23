import {
  commands,
  type Car,
  type CarBrand,
  type Client,
  type Service,
} from "$lib/bindings";

class AppStore {
  version: string = $state("0.0.0");
  clients: Client[] = $state([]);
  cars: Car[] = $state([]);
  car_brands: CarBrand[] = $state([]);
  services: Service[] = $state([]);
  documents = $state([]);
  loading = $state(false);
  error: string | null = $state(null);

  async loadAll() {
    this.loading = true;
    this.error = null;
    try {
      let res1 = await commands.fetchClients();
      if (res1.status === "ok") {
        this.clients = res1.data;
      } else {
        this.error = String(res1.error);
      }
      let res2 = await commands.fetchCarBrands();
      if (res2.status === "ok") {
        this.car_brands = res2.data;
      } else {
        this.error = String(res2.error);
      }
      let res3 = await commands.fetchCars();
      if (res3.status === "ok") {
        this.cars = res3.data;
      } else {
        this.error = String(res3.error);
      }
      let res4 = await commands.fetchServices();
      if (res4.status === "ok") {
        this.services = res4.data;
      } else {
        this.error = String(res4.error);
      }
      commands.getAppVersion().then((res) => {
        this.version = res;
      });
    } catch (e) {
      this.error = "Failed to communicate with backend process.";
    } finally {
      this.loading = false;
    }
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

  removeClient(id: number) {
    this.clients = this.clients.filter((client) => client.id !== id);
  }

  removeCar(id: number) {
    this.cars = this.cars.filter((car) => car.id !== id);
  }

  removeService(id: number) {
    this.services = this.services.filter((service) => service.id !== id);
  }
}

export const appStore = new AppStore();
