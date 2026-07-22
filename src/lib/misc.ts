import Sqids from "sqids";

let date_locale = "uk-UA";
const sqids = new Sqids({
  alphabet: "65OVH9R8SPELUNT71GYX4AJ3IDCBQZW2FKM",
  minLength: 10,
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

export function sqidsEncode(value: number[]) {
  return sqids.encode(value);
}
