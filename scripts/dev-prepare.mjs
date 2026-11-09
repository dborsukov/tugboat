// Local dev setup, run by pnpm through the prepare script.
// Git hooks and the cargo-sort plugin are skipped in CI.
import { execFileSync } from "node:child_process";

function works(command, args) {
  try {
    execFileSync(command, args, { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

if (process.env.CI) {
  process.exit(0);
}

if (!works("cargo", ["sort", "--version"])) {
  execFileSync("cargo", ["install", "cargo-sort"], { stdio: "inherit" });
}

if (works("pre-commit", ["--version"])) {
  execFileSync("pre-commit", ["install"], { stdio: "inherit" });
} else {
  console.warn(
    "pre-commit is not installed, so the git hooks were not set up. " +
      "Install it with `pip install pre-commit`, then run `pre-commit install`.",
  );
}
