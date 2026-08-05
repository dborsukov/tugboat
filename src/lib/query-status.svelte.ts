type StatusSource = {
  isSuccess: boolean;
  isPending: boolean;
  isError: boolean;
  error: unknown;
};

export function createQueryStatus(queries: () => StatusSource[]) {
  const isReady = $derived(queries().every((q) => q.isSuccess));
  const isLoading = $derived(queries().some((q) => q.isPending));
  const isError = $derived(queries().some((q) => q.isError));
  const firstError = $derived(queries().find((q) => q.isError)?.error);

  return {
    get isReady() {
      return isReady;
    },
    get isLoading() {
      return isLoading;
    },
    get isError() {
      return isError;
    },
    get firstError() {
      return firstError;
    },
  };
}
