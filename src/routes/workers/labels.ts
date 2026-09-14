import type { Worker } from "$lib/bindings";

export function workerFullName(worker: Worker): string {
  return [worker.surname, worker.firstName, worker.patronymic]
    .map((part) => part?.trim())
    .filter((part): part is string => !!part)
    .join(" ");
}

export function workerShortName(worker: Worker): string {
  const initials = [worker.firstName, worker.patronymic]
    .map((part) => part?.trim().charAt(0))
    .filter((initial): initial is string => !!initial)
    .map((initial) => `${initial}.`)
    .join(" ");
  return [worker.surname.trim(), initials]
    .filter((part) => part.length > 0)
    .join(" ");
}
