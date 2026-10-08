export type Platform = "macos" | "windows" | "linux";

function detectPlatform(): Platform {
  const marked = document.documentElement.dataset.platform;
  if (marked === "macos" || marked === "windows" || marked === "linux") {
    return marked;
  }

  const userAgent = navigator.userAgent;
  if (userAgent.includes("Mac OS X")) return "macos";
  if (userAgent.includes("Windows")) return "windows";
  return "linux";
}

/** Fixed for the lifetime of the window. */
export const platform: Platform = detectPlatform();

export const isMacOS = platform === "macos";
