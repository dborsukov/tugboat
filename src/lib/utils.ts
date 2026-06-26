import { clsx, type ClassValue } from "clsx";
import Sqids from "sqids";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any }
  ? Omit<T, "children">
  : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};

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
