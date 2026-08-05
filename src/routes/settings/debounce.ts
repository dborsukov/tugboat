export function debounce<Args extends unknown[]>(
  fn: (...args: Args) => void,
  delay: number,
) {
  let timer: ReturnType<typeof setTimeout> | undefined;

  function run(...args: Args) {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  }

  run.flush = (...args: Args) => {
    clearTimeout(timer);
    fn(...args);
  };

  run.cancel = () => clearTimeout(timer);

  return run;
}
