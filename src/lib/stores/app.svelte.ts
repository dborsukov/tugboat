import { commands, type Client } from "$lib/bindings";

class AppStore {
  version: string = $state("0.0.0");
  clients: Client[] = $state([]);
  cars = $state([]);
  documents = $state([]);
  loading = $state(false);
  error: string | null = $state(null);

  async loadAll() {
    this.loading = true;
    this.error = null;
    try {
      let res = await commands.fetchClients();
      if (res.status === "ok") {
        this.clients = res.data;
      } else {
        this.error = String(res.error);
      }
    } catch (e) {
      this.error = "Failed to communicate with backend process.";
    } finally {
      this.loading = false;
    }
    commands.getAppVersion().then((res) => {
      this.version = res;
    });
  }

  createClient(client: Client) {
    console.log(`Created: ${client}`);
  }

  updateClient(client: Client) {
    console.log(`Updated: ${client}`);
  }

  removeClient(id: number) {
    this.clients = this.clients.filter((client) => client.id !== id);
  }
}

export const appStore = new AppStore();
