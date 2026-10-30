import { goto } from "$app/navigation";
import { page } from "$app/state";

import { getSetupIssues, type SetupIssue } from "$lib/states/setup-issues";

const FLASH_CLASS = "settings-target-flash";
const FOCUS_TIMEOUT_MS = 2500;
const FOCUS_POLL_MS = 50;

/** Thrown by mutation guards so a background call cannot create a document. */
export class SetupBlockedError extends Error {
  constructor() {
    super("Спочатку заповніть дані бізнесу");
    this.name = "SetupBlockedError";
  }
}

class SetupAttentionState {
  issue = $state<SetupIssue | null>(null);

  show(issue: SetupIssue) {
    this.issue = issue;
  }

  dismiss() {
    this.issue = null;
  }
}

export const setupAttention = new SetupAttentionState();

/** Checks the single source of truth before a document is produced. */
export function blockDocumentGeneration(): boolean {
  const issue = getSetupIssues().find(
    (candidate) => candidate.key === "business_info",
  );
  if (!issue) return false;

  setupAttention.show(issue);
  return true;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function openSetupIssue(issue: SetupIssue) {
  setupAttention.dismiss();

  if (page.url.pathname !== issue.page) {
    await goto(issue.page);
  }

  const targetId = issue.fields[0]?.targetId;
  const deadline = Date.now() + FOCUS_TIMEOUT_MS;

  while (Date.now() < deadline) {
    const target = targetId ? document.getElementById(targetId) : null;

    if (target instanceof HTMLElement) {
      const section = document.getElementById(issue.section);
      if (section) {
        section.classList.add(FLASH_CLASS);
        setTimeout(() => section.classList.remove(FLASH_CLASS), 2000);
      }

      target.scrollIntoView({ behavior: "smooth", block: "center" });
      target.focus({ preventScroll: true });
      return;
    }

    await sleep(FOCUS_POLL_MS);
  }
}
