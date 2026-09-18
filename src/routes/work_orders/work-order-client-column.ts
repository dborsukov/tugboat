import type { WorkOrderDetailsDto } from "$lib/bindings";

export type WorkOrderClientRole = "owner" | "payer";

export type WorkOrderClientLine = {
  role: WorkOrderClientRole;
  label: string;
  name: string | null;
};

export type WorkOrderClientColumn = {
  mainName: string | null;
  others: WorkOrderClientLine[];
  loading: boolean;
};

export function buildWorkOrderClientColumn(
  details: WorkOrderDetailsDto,
  clientName: (clientId: number) => string | null,
  clientsLoaded: boolean,
): WorkOrderClientColumn {
  const owner = details.client;
  const { customerClientId, payerClientId } = details.workOrder;
  const customer =
    customerClientId != null && customerClientId !== owner.id
      ? customerClientId
      : null;
  const others: WorkOrderClientLine[] = [];

  if (customer != null) {
    others.push({ role: "owner", label: "Власник", name: owner.name });
    if (payerClientId !== customer) {
      others.push({
        role: "payer",
        label: "Платник",
        name: payerClientId == null ? owner.name : clientName(payerClientId),
      });
    }
  } else if (payerClientId != null && payerClientId !== owner.id) {
    others.push({
      role: "payer",
      label: "Платник",
      name: clientName(payerClientId),
    });
  }

  return {
    mainName: customer != null ? clientName(customer) : owner.name,
    others,
    loading: !clientsLoaded,
  };
}
