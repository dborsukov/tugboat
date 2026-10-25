import type { CarDetailsDto, ClientListItem } from "$lib/bindings";
import { plateSearchKey } from "$lib/plate";

export type PickerResult =
  | {
      kind: "car";
      id: number;
      rank: number;
      car: CarDetailsDto;
      owner: ClientListItem | null;
    }
  | {
      kind: "client";
      id: number;
      rank: number;
      client: ClientListItem;
    };

export function normalizePlateKey(value: string | null | undefined) {
  return plateSearchKey(value);
}

export function normalizeVinKey(value: string | null | undefined) {
  return (value ?? "").toUpperCase().replace(/[\s-]/g, "");
}

export function phoneDigits(value: string | null | undefined) {
  return (value ?? "").replace(/\D/g, "");
}

export function matchesWords(text: string, query: string) {
  const haystack = text.toLowerCase();
  return query
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((word) => haystack.includes(word));
}

export function ownerPhone(client: ClientListItem | null | undefined) {
  return client?.phone ?? client?.contactPhone ?? "";
}

function matchesPhone(
  client: ClientListItem | null | undefined,
  query: string,
) {
  const digits = phoneDigits(query);
  if (digits.length < 3) return false;
  return phoneDigits(ownerPhone(client)).includes(digits);
}

function isActiveCar(car: CarDetailsDto) {
  return car.carDto.car.archivedAt == null && car.client.archivedAt == null;
}

function inScope(car: CarDetailsDto, scopedClientId: number | null) {
  return scopedClientId == null || car.carDto.car.clientId === scopedClientId;
}

/** Cars that can be picked, newest first, regardless of the search. */
export function pickableCars(
  cars: CarDetailsDto[],
  clients: ClientListItem[],
  scopedClientId: number | null,
) {
  const clientsById = new Map(clients.map((client) => [client.id, client]));
  return cars
    .filter((car) => isActiveCar(car) && inScope(car, scopedClientId))
    .map((car) => ({
      car,
      owner: clientsById.get(car.carDto.car.clientId) ?? null,
    }));
}

export function recentCars(
  cars: CarDetailsDto[],
  clients: ClientListItem[],
  recentCarIds: number[],
  scopedClientId: number | null,
  limit = 8,
) {
  const available = pickableCars(cars, clients, scopedClientId);
  const byId = new Map(available.map((item) => [item.car.carDto.car.id, item]));
  const seen = new Set<number>();
  const ordered = [];

  for (const id of recentCarIds) {
    const item = byId.get(id);
    if (!item || seen.has(id)) continue;
    ordered.push(item);
    seen.add(id);
  }

  if (ordered.length > 0) return ordered.slice(0, limit);

  return available
    .sort(
      (a, b) =>
        new Date(b.car.carDto.car.createdAt).getTime() -
        new Date(a.car.carDto.car.createdAt).getTime(),
    )
    .slice(0, limit);
}

export function searchCars(
  cars: CarDetailsDto[],
  clients: ClientListItem[],
  query: string,
  scopedClientId: number | null,
  limit = 8,
): PickerResult[] {
  const text = query.trim();
  if (text === "") return [];

  const available = pickableCars(cars, clients, scopedClientId);
  const normalized = text.toUpperCase().replace(/[\s-]/g, "");
  const lower = text.toLowerCase();
  const results: PickerResult[] = [];

  for (const { car, owner } of available) {
    const plate = car.carDto.car.plateSearchKey ?? "";
    const vin = normalizeVinKey(car.carDto.car.vin);
    const brandModel = `${car.carDto.brand.name} ${car.carDto.car.model}`
      .trim()
      .toLowerCase();
    const ownerName = owner?.name ?? car.client.name;

    let rank: number | null = null;

    if (plate !== "" && plate === normalized) {
      rank = 0;
    } else if (
      vin !== "" &&
      normalized.length >= 2 &&
      vin.startsWith(normalized)
    ) {
      rank = 1;
    } else if (plate !== "" && plate.includes(normalized)) {
      rank = 2;
    } else if (brandModel.startsWith(lower)) {
      rank = 3;
    } else if (brandModel.includes(lower)) {
      rank = 4;
    } else if (matchesWords(ownerName, text)) {
      rank = 5;
    } else if (matchesPhone(owner ?? null, text)) {
      rank = 6;
    }

    if (rank == null) continue;
    results.push({
      kind: "car",
      id: car.carDto.car.id,
      rank,
      car,
      owner,
    });
  }

  const activeCarClients = new Set(
    available.map((item) => item.car.carDto.car.clientId),
  );
  for (const client of clients) {
    if (client.archivedAt != null) continue;
    if (scopedClientId != null && client.id !== scopedClientId) continue;
    if (activeCarClients.has(client.id)) continue;

    let rank: number | null = null;
    if (matchesWords(client.name, text)) rank = 5;
    else if (matchesPhone(client, text)) rank = 6;
    if (rank == null) continue;

    results.push({ kind: "client", id: client.id, rank, client });
  }

  return results
    .sort((a, b) => {
      if (a.rank !== b.rank) return a.rank - b.rank;
      const nameA =
        a.kind === "car"
          ? `${a.car.carDto.brand.name} ${a.car.carDto.car.model}`
          : a.client.name;
      const nameB =
        b.kind === "car"
          ? `${b.car.carDto.brand.name} ${b.car.carDto.car.model}`
          : b.client.name;
      return nameA.localeCompare(nameB, "uk");
    })
    .slice(0, limit);
}

/** The single client the bottom "new car for…" action can safely target. */
export function singleClientOfResults(
  results: PickerResult[],
  clients: ClientListItem[],
  query: string,
  scopedClientId: number | null,
): ClientListItem | null {
  if (scopedClientId != null) {
    return clients.find((client) => client.id === scopedClientId) ?? null;
  }

  const ids = new Set<number>();
  for (const result of results) {
    ids.add(
      result.kind === "car" ? result.car.carDto.car.clientId : result.client.id,
    );
  }
  if (ids.size === 1) {
    const id = [...ids][0];
    return clients.find((client) => client.id === id) ?? null;
  }

  const text = query.trim();
  if (text === "") return null;

  const matches = clients.filter(
    (client) =>
      client.archivedAt == null &&
      (matchesWords(client.name, text) || matchesPhone(client, text)),
  );
  return matches.length === 1 ? matches[0] : null;
}
