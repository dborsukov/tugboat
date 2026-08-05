export class TableActionState<TId = number | string> {
  editId = $state<TId | null>(null);
  deleteId = $state<TId | null>(null);

  isMutationOpen = $state(false);
  isDeleteOpen = $state(false);

  openCreate = () => {
    this.editId = null;
    this.isMutationOpen = true;
  };

  openEdit = (id: TId) => {
    this.editId = id;
    this.isMutationOpen = true;
  };

  openDelete = (id: TId) => {
    this.deleteId = id;
    this.isDeleteOpen = true;
  };

  resetEdit = () => {
    this.isMutationOpen = false;
    this.editId = null;
  };

  resetDelete = () => {
    this.isDeleteOpen = false;
    this.deleteId = null;
  };
}
