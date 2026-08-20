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

let date_locale = "uk-UA";

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
