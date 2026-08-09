export type CreateOrEditProps<TCreate, TEdit> =
  | ({ mode: "create" } & TCreate)
  | ({ mode: "edit" } & TEdit);
