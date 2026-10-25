<script lang="ts">
  import { toast } from "svelte-sonner";

  import { goto } from "$app/navigation";

  import { interpretQuery } from "$lib/components/sheet/query-prefill";
  import SheetHost from "$lib/components/sheet/sheet-host.svelte";

  import type { QuickOrderContext } from "$lib/states/quick-order.svelte";

  import CarSheetContent from "../cars/car-sheet-content.svelte";
  import ClientSheetContent from "../clients/client-sheet-content.svelte";
  import { createClientDetailsQuery } from "../clients/queries";
  import WorkOrderSheetContent from "./work-order-sheet-content.svelte";

  let {
    open = $bindable(false),
    context = {},
    requestId = 0,
  }: {
    open: boolean;
    context: QuickOrderContext;
    requestId: number;
  } = $props();

  type Step = "order" | "client" | "car";
  type CarOrigin = "picker" | "chain" | null;

  let step = $state<Step>("order");
  let orderSession = $state(0);
  let appliedRequestId = $state(-1);

  let selectedCarId = $state<number | null>(null);
  let pickerQuery = $state("");
  let scopedClientId = $state<number | null>(null);

  let carOwnerId = $state<number | null>(null);
  let carOwnerName = $state("");
  let carInitialQuery = $state("");
  let carStepLabel = $state<string | undefined>(undefined);
  let carOrigin = $state<CarOrigin>(null);
  let carReturnSelectedId = $state<number | null>(null);

  let clientInitialQuery = $state("");
  let previousPickerQuery = $state("");
  let previousScopedClientId = $state<number | null>(null);
  let contextQueryApplied = $state(false);

  let pendingClientId = $state<number | null>(null);
  let newClientId = $state<number | null>(null);
  let newClientName = $state("");

  const contextClientQuery = createClientDetailsQuery(() =>
    open && context.clientId != null ? context.clientId : null,
  );
  const pendingClientQuery = createClientDetailsQuery(() => pendingClientId);

  function resetFlow() {
    const carId = context.carId ?? null;
    const clientId = context.clientId ?? null;

    step = "order";
    selectedCarId = carId;
    scopedClientId = clientId;
    pickerQuery = "";
    previousPickerQuery = "";
    previousScopedClientId = clientId;

    carOwnerId = null;
    carOwnerName = "";
    carInitialQuery = "";
    carStepLabel = undefined;
    carOrigin = null;
    carReturnSelectedId = carId;
    clientInitialQuery = "";

    pendingClientId = null;
    newClientId = null;
    newClientName = "";
    contextQueryApplied = false;

    orderSession += 1;
  }

  $effect(() => {
    if (!open) return;
    if (appliedRequestId === requestId) return;
    appliedRequestId = requestId;
    resetFlow();
  });

  // On a client page the picker starts scoped to that client. Once their name
  // is known, put it in the search so the client is visibly the subject.
  $effect(() => {
    if (!open || contextQueryApplied) return;
    const data = contextClientQuery.data;
    if (!data) return;
    contextQueryApplied = true;
    if (step === "order" && selectedCarId == null && pickerQuery === "") {
      pickerQuery = data.client.name;
    }
  });

  $effect(() => {
    const data = pendingClientQuery.data;
    if (!data || data.client.id !== pendingClientId) return;
    newClientName = data.client.name;
    if (carOwnerId === data.client.id) carOwnerName = data.client.name;
  });

  function clientNameFromQuery(query: string) {
    const interpreted = interpretQuery(query);
    if (interpreted.kind === "text") return interpreted.words[0] ?? "";
    return "";
  }

  function openClientStep() {
    previousPickerQuery = pickerQuery;
    previousScopedClientId = scopedClientId;
    clientInitialQuery = pickerQuery;
    step = "client";
  }

  function handleClientCreated(id: number, name?: string) {
    pendingClientId = id;
    newClientId = id;
    newClientName = name ?? newClientName;
    carOwnerId = id;
    carOwnerName =
      newClientName || clientNameFromQuery(pickerQuery) || "Клієнт";
    carInitialQuery = pickerQuery;
    carOrigin = "chain";
    carReturnSelectedId = selectedCarId;
    carStepLabel = "Крок 2 з 2";
    step = "car";
  }

  function backFromClient() {
    step = "order";
    pickerQuery = previousPickerQuery;
    scopedClientId = previousScopedClientId;
  }

  function openCarStepForClient(clientId: number, clientName: string) {
    previousPickerQuery = pickerQuery;
    previousScopedClientId = scopedClientId;
    carOwnerId = clientId;
    carOwnerName = clientName;
    carOrigin = "picker";
    carReturnSelectedId = selectedCarId;
    carInitialQuery = pickerQuery === clientName ? "" : pickerQuery;
    carStepLabel = undefined;
    step = "car";
  }

  function handleCarCreated(id: number) {
    selectedCarId = id;
    pickerQuery = "";
    step = "order";
    if (carOrigin === "chain" && newClientId != null) {
      scopedClientId = newClientId;
    }
    carOrigin = null;
    carStepLabel = undefined;
    carOwnerId = null;
    carOwnerName = "";
    carInitialQuery = "";
  }

  function backFromCar() {
    step = "order";
    selectedCarId = carReturnSelectedId;
    if (carOrigin === "chain" && newClientId != null) {
      // The client was already created; keep them and go back to their card
      // in the picker instead of throwing the work away.
      scopedClientId = newClientId;
      pickerQuery =
        newClientName || clientNameFromQuery(previousPickerQuery) || "";
    } else {
      scopedClientId = previousScopedClientId;
      pickerQuery = previousPickerQuery;
    }
    carOrigin = null;
    carStepLabel = undefined;
    carOwnerId = null;
    carOwnerName = "";
    carInitialQuery = "";
  }

  // Switching the single sheet from the client step to the car step makes the
  // back link the first focusable element, so the browser/focus scope can land
  // on it. Move focus to the first car field once the step has settled.
  $effect(() => {
    if (!open || step !== "car") return;
    let cancelled = false;
    let attempts = 0;
    let timer: ReturnType<typeof setTimeout>;

    const focusFirstCarField = () => {
      if (cancelled) return;
      const active = document.activeElement;
      if (active instanceof HTMLElement && active.closest("form")) return;

      const target =
        document.querySelector<HTMLElement>("#car-brand-select") ?? null;
      if (target && target.getClientRects().length > 0) {
        target.focus();
        return;
      }
      if (++attempts < 4) timer = setTimeout(focusFirstCarField, 80);
    };

    timer = setTimeout(focusFirstCarField, 220);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  function handleOrderCreated(id: string) {
    open = false;
    toast.success("Наряд-замовлення створено", {
      position: "bottom-left",
    });
    void goto(`/work_orders/${id}`);
  }
</script>

<SheetHost bind:open autofocus>
  <div class={step === "order" ? "flex min-h-0 flex-1 flex-col" : "hidden"}>
    <WorkOrderSheetContent
      active={open && step === "order"}
      session={orderSession}
      mode="create"
      initialCarId={context.carId ?? null}
      allowPicker
      allowCreateActions
      {scopedClientId}
      bind:selectedCarId
      bind:pickerQuery
      onCancel={() => (open = false)}
      onCreated={handleOrderCreated}
      onNewCarForClient={openCarStepForClient}
      onNewClientAndCar={openClientStep}
    />
  </div>

  {#if step === "client"}
    <div class="flex min-h-0 flex-1 flex-col">
      <ClientSheetContent
        active={open}
        initialQuery={clientInitialQuery}
        stepLabel="Крок 1 з 2"
        backLabel="Назад до замовлення"
        onBack={backFromClient}
        onCancel={backFromClient}
        onCreated={handleClientCreated}
      />
    </div>
  {/if}

  {#if step === "car"}
    <div class="flex min-h-0 flex-1 flex-col">
      <CarSheetContent
        active={open}
        ownerClientId={carOwnerId}
        ownerName={carOwnerName}
        initialQuery={carInitialQuery}
        stepLabel={carStepLabel}
        backLabel="Назад до замовлення"
        onBack={backFromCar}
        onCancel={backFromCar}
        onCreated={handleCarCreated}
      />
    </div>
  {/if}
</SheetHost>
