export class TableActionState<TId = number | string> {
  editId = $state<TId | null>(null);
  deleteId = $state<TId | null>(null);
  archiveId = $state<TId | null>(null);
  bulkDeleteIds = $state<TId[]>([]);
  bulkArchiveIds = $state<TId[]>([]);

  isMutationOpen = $state(false);
  isDeleteOpen = $state(false);
  isBulkDeleteOpen = $state(false);
  isArchiveOpen = $state(false);
  isBulkArchiveOpen = $state(false);

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

  openBulkDelete = (ids: TId[]) => {
    this.bulkDeleteIds = ids;
    this.isBulkDeleteOpen = true;
  };

  openArchive = (id: TId) => {
    this.archiveId = id;
    this.isArchiveOpen = true;
  };

  openBulkArchive = (ids: TId[]) => {
    this.bulkArchiveIds = ids;
    this.isBulkArchiveOpen = true;
  };

  resetEdit = () => {
    this.isMutationOpen = false;
    this.editId = null;
  };

  resetDelete = () => {
    this.isDeleteOpen = false;
    this.deleteId = null;
  };

  resetBulkDelete = () => {
    this.isBulkDeleteOpen = false;
    this.bulkDeleteIds = [];
  };

  resetArchive = () => {
    this.isArchiveOpen = false;
    this.archiveId = null;
  };

  resetBulkArchive = () => {
    this.isBulkArchiveOpen = false;
    this.bulkArchiveIds = [];
  };
}
