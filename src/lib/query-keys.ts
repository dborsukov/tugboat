export const keys = {
  clients: {
    all: ["clients"] as const,
    list: () => [...keys.clients.all, "list"] as const,
    count: () => [...keys.clients.all, "count"] as const,
    details: () => [...keys.clients.all, "detail"] as const,
    detail: (id: string | number) => [...keys.clients.details(), id] as const,
    deleteImpact: (id: number) =>
      [...keys.clients.all, "delete-impact", id] as const,
  },
  cars: {
    all: ["cars"] as const,
    list: () => [...keys.cars.all, "list"] as const,
    listDetails: () => [...keys.cars.all, "list-details"] as const,
    count: () => [...keys.cars.all, "count"] as const,
    details: () => [...keys.cars.all, "detail"] as const,
    detail: (id: string | number) => [...keys.cars.details(), id] as const,
    deleteImpact: (id: number) =>
      [...keys.cars.all, "delete-impact", id] as const,
  },
  carBrand: {
    all: ["carBrand"] as const,
    list: () => [...keys.carBrand.all, "list"] as const,
    custom: () => [...keys.carBrand.all, "custom"] as const,
  },
  carModel: {
    all: ["carModel"] as const,
    list: (brandId: number) => [...keys.carModel.all, "list", brandId] as const,
  },
  workOrders: {
    all: ["work-orders"] as const,
    list: () => [...keys.workOrders.all, "list"] as const,
    count: () => [...keys.workOrders.all, "count"] as const,
    laborItems: () => [...keys.workOrders.all, "labor-item"] as const,
    laborItem: (id: string | number) =>
      [...keys.workOrders.laborItems(), id] as const,
    partItems: () => [...keys.workOrders.all, "part-items"] as const,
    partItem: (id: string | number) =>
      [...keys.workOrders.partItems(), id] as const,
    details: () => [...keys.workOrders.all, "detail"] as const,
    detail: (id: string | number) =>
      [...keys.workOrders.details(), id] as const,
    deleteImpact: (id: string) =>
      [...keys.workOrders.all, "delete-impact", id] as const,
    documents: (id: string | number) =>
      [...keys.workOrders.all, "documents", id] as const,
    vat: (id: string | number) => [...keys.workOrders.all, "vat", id] as const,
  },
  services: {
    all: ["services"] as const,
    list: () => [...keys.services.all, "list"] as const,
    count: () => [...keys.services.all, "count"] as const,
    details: () => [...keys.services.all, "detail"] as const,
    detail: (id: string | number) => [...keys.services.details(), id] as const,
  },
  workers: {
    all: ["workers"] as const,
    list: () => [...keys.workers.all, "list"] as const,
    count: () => [...keys.workers.all, "count"] as const,
    details: () => [...keys.workers.all, "detail"] as const,
    detail: (id: string | number) => [...keys.workers.details(), id] as const,
  },
  expenses: {
    all: ["expenses"] as const,
    list: () => [...keys.expenses.all, "list"] as const,
    count: () => [...keys.expenses.all, "count"] as const,
    details: () => [...keys.expenses.all, "detail"] as const,
    detail: (id: string | number) => [...keys.expenses.details(), id] as const,
    vat: (id: string | number) => [...keys.expenses.all, "vat", id] as const,
  },
  expenseCategories: {
    all: ["expense-categories"] as const,
    list: () => [...keys.expenseCategories.all, "list"] as const,
  },
  reports: {
    all: ["reports"] as const,
    list: () => [...keys.reports.all, "list"] as const,
  },
  settings: {
    all: ["settings"] as const,
  },
  recent: {
    all: ["recent"] as const,
    list: () => [...keys.recent.all, "list"] as const,
  },
  dashboard: {
    all: ["dashboard"] as const,
    module: (id: string, period?: unknown) =>
      period === undefined
        ? ([...keys.dashboard.all, id] as const)
        : ([...keys.dashboard.all, id, period] as const),
  },
  version: ["version"] as const,
};
