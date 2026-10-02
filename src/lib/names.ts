type NamePart = string | null | undefined;

export function joinPib(...parts: NamePart[]): string {
  return parts
    .map((part) => part?.trim())
    .filter((part): part is string => !!part)
    .join(" ");
}

/**
 * A person's name in short form: "Шевченко Т.Г.".
 * Mirrors `pib_initials` in src-tauri/src/documents/model.rs for preview
 */
export function pibInitials(
  surname: NamePart,
  firstName: NamePart,
  patronymic: NamePart,
): string {
  const trimmedSurname = surname?.trim() ?? "";
  if (!trimmedSurname) {
    return joinPib(firstName, patronymic);
  }

  const initials = [firstName, patronymic]
    .map((part) => part?.trim())
    .filter((part): part is string => !!part)
    // A whole code point
    .map((part) => `${Array.from(part)[0]}.`)
    .join("");

  return initials ? `${trimmedSurname} ${initials}` : trimmedSurname;
}
