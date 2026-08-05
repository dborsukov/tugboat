export const keys = {
  clients: {
    all: ["clients"] as const,
    list: () => [...keys.clients.all, "list"] as const,
    count: () => [...keys.clients.all, "count"] as const,
    details: () => [...keys.clients.all, "detail"] as const,
    detail: (id: string | number) => [...keys.clients.details(), id] as const,
  },
  cars: {
    all: ["cars"] as const,
    list: () => [...keys.cars.all, "list"] as const,
    count: () => [...keys.cars.all, "count"] as const,
  },
  carBrand: ["carBrand"] as const,
  workOrders: {
    all: ["work-orders"] as const,
    lists: () => [...keys.workOrders.all, "list"] as const,
    count: () => [...keys.workOrders.all, "count"] as const,
    laborItems: () => [...keys.workOrders.all, "labor-items"] as const,
    partItems: () => [...keys.workOrders.all, "part-items"] as const,
    totals: () => [...keys.workOrders.all, "totals"] as const,
  },
  services: {
    all: ["services"] as const,
    list: () => [...keys.services.all, "list"] as const,
    count: () => [...keys.services.all, "count"] as const,
    details: () => [...keys.clients.all, "detail"] as const,
    detail: (id: string | number) => [...keys.clients.details(), id] as const,
  },
  settings: {
    all: ["settings"] as const,
  },
};
