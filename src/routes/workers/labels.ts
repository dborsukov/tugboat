import type { Worker } from "$lib/bindings";
import { joinPib, pibInitials } from "$lib/names";

export function workerFullName(worker: Worker): string {
  return joinPib(worker.surname, worker.firstName, worker.patronymic);
}

export function workerShortName(worker: Worker): string {
  return pibInitials(worker.surname, worker.firstName, worker.patronymic);
}
