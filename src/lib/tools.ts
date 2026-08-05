import Sqids from "sqids";

type SpectaResult<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

export async function unwrap<T, E>(
  promise: Promise<SpectaResult<T, E>>,
): Promise<T> {
  const result = await promise;
  if (result.status === "error") {
    throw result.error;
  }
  return result.data;
}

let squids_counter = 0;
let date_locale = "uk-UA";
const sqids = new Sqids({
  alphabet: "65VH9R8SPELUNT71GYX4AJ3DCBZW2FKM",
  minLength: 6,
});

export function prettifyIsoDateString(
  iso_string: string,
  type: "short" | "long" | "datetime",
) {
  if (type === "short") {
    return new Date(iso_string).toLocaleDateString(date_locale, {
      month: "numeric",
      day: "numeric",
      year: "numeric",
    });
  }
  if (type === "long") {
    return new Date(iso_string).toLocaleDateString(date_locale, {
      month: "long",
      day: "numeric",
      year: "numeric",
    });
  }
  if (type === "datetime") {
    return new Date(iso_string).toLocaleString(date_locale, {
      month: "long",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "numeric",
    });
  }
}

export function sqidsEncode(value: number) {
  // Reset counter every 1000 items to keep numbers lean
  squids_counter = (squids_counter + 1) % 1000;
  return sqids.encode([value, squids_counter]);
}
