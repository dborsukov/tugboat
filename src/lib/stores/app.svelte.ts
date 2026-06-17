import { commands, type Car, type CarBrand, type Client } from "$lib/bindings";

class AppStore {
  version: string = $state("0.0.0");
  clients: Client[] = $state([]);
  cars: Car[] = $state([]);
  car_brands: CarBrand[] = $state([]);
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
      commands.getAppVersion().then((res) => {
        this.version = res;
      });
    } catch (e) {
      this.error = "Failed to communicate with backend process.";
    } finally {
      this.loading = false;
    }
  }

  removeClient(id: number) {
    this.clients = this.clients.filter((client) => client.id !== id);
  }

  removeCar(id: number) {
    this.cars = this.cars.filter((car) => car.id !== id);
  }
}

export const appStore = new AppStore();
