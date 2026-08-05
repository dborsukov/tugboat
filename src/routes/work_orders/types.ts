import type {
  Car,
  CarBrand,
  Client,
  WorkOrder,
  WorkOrderLaborItem,
  WorkOrderPartItem,
  WorkOrderTotal,
} from "$lib/bindings";

export type WorkOrderListItem = WorkOrder & {
  car: Car;
  carBrand: CarBrand;
  client: Client;
  laborItems: WorkOrderLaborItem[];
  partsItems: WorkOrderPartItem[];
  laborTotalCents: number;
  partsTotalCents: number;
  subtotalCents: number;
  grandTotalCents: number;
};

export function buildWorkOrderListItems(
  workOrders: WorkOrder[],
  cars: Car[],
  carBrands: CarBrand[],
  clients: Client[],
  laborItems: WorkOrderLaborItem[],
  partItems: WorkOrderPartItem[],
  totals: WorkOrderTotal[],
  filter?: (workOrder: WorkOrder) => boolean,
): WorkOrderListItem[] {
  const totalsByWorkOrderId = new Map(totals.map((t) => [t.workOrderId, t]));
  const orders = filter ? workOrders.filter(filter) : workOrders;

  const items: WorkOrderListItem[] = [];
  for (const order of orders) {
    const car = cars.find((c) => c.id === order.carId);
    const carBrand = carBrands.find((b) => b.id === car?.brandId);
    const client = clients.find((c) => c.id === car?.clientId);

    if (!car || !carBrand || !client) {
      console.error(
        `Work order ${order.id} references a missing car/brand/client — skipping.`,
      );
      continue;
    }

    const orderTotals = totalsByWorkOrderId.get(order.id);
    items.push({
      ...order,
      car,
      carBrand,
      client,
      laborItems: laborItems.filter((i) => i.workOrderId === order.id),
      partsItems: partItems.filter((i) => i.workOrderId === order.id),
      laborTotalCents: orderTotals?.laborTotalCents ?? 0,
      partsTotalCents: orderTotals?.partsTotalCents ?? 0,
      subtotalCents: orderTotals?.subtotalCents ?? 0,
      grandTotalCents: orderTotals?.grandTotalCents ?? 0,
    });
  }
  return items;
}
