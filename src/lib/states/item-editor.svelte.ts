export type ItemEditorOptions<T extends object> = {
  /** Reactive getter for the current list of items. */
  items: () => T[];
  /** Build a fresh draft used when creating a new item. */
  makeEmpty: () => T;
  /**
   * Validate a draft against the rest of the list (the row being edited is
   * excluded, so per-row rules like "no duplicate names" work across rows).
   * Return a map of field name -> error message (or undefined when valid).
   */
  validate: (entry: T, others: T[]) => Record<string, string | undefined>;
  /** Persist a new list — write it into state and run the save mutation. */
  commit: (items: T[]) => void;
  /** Optional normalization applied to an entry right before it is committed. */
  transform?: (entry: T) => T;
};

/**
 * Generic create/edit/remove state for a list of items edited through a modal
 * dialog. Owns the dialog visibility, the index being edited (null while
 * creating), the in-progress draft, and live validation against the rest of
 * the list.
 */
export function createItemEditor<T extends object>(
  options: ItemEditorOptions<T>,
) {
  let isOpen = $state(false);
  /** null = creating a new item, otherwise the index being edited */
  let editingIndex = $state<number | null>(null);
  /** The in-progress entry; replaced with a fresh draft on every open. */
  let draft = $state<T>(options.makeEmpty());

  const others = $derived(
    editingIndex === null
      ? options.items()
      : options.items().filter((_, i) => i !== editingIndex),
  );
  const errors = $derived(options.validate(draft, others));
  const hasErrors = $derived(Object.values(errors).some(Boolean));

  function openCreate() {
    editingIndex = null;
    draft = options.makeEmpty();
    isOpen = true;
  }

  function openEdit(index: number) {
    editingIndex = index;
    draft = { ...options.items()[index] };
    isOpen = true;
  }

  function save() {
    if (hasErrors) return;
    const entry = options.transform
      ? options.transform({ ...draft })
      : { ...draft };
    const items = options.items();
    options.commit(
      editingIndex === null
        ? [...items, entry]
        : items.map((v, i) => (i === editingIndex ? entry : v)),
    );
    isOpen = false;
  }

  function remove(index: number) {
    options.commit(options.items().filter((_, i) => i !== index));
  }

  function close() {
    isOpen = false;
  }

  return {
    get isOpen() {
      return isOpen;
    },
    set isOpen(value: boolean) {
      isOpen = value;
    },
    get editingIndex() {
      return editingIndex;
    },
    get draft() {
      return draft;
    },
    get errors() {
      return errors;
    },
    get hasErrors() {
      return hasErrors;
    },
    openCreate,
    openEdit,
    save,
    remove,
    close,
  };
}
