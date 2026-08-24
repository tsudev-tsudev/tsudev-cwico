/**
 * The IPC boundary.
 *
 * Every call the front end makes to the engine goes through this module, and
 * nothing else imports `@tauri-apps/api` directly. That gives one place to
 * translate errors, and one place to fall back to fixtures when the UI runs
 * in a plain browser - which is how the interface gets developed and reviewed
 * without a Windows machine in the loop.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AboutInfo,
  ApiError,
  PlanOptions,
  ProgressEvent,
  RemovalPlan,
  RunReport,
  ScanOptions,
  ScanReport,
  Selection,
  TweakCatalog,
  TweakOutcome,
  UpdateProgress,
  UpdateStatus,
} from "./types";
import * as fixtures from "./fixtures";

const PROGRESS_EVENT = "cwico://progress";
const UPDATE_PROGRESS_EVENT = "cwico://update-progress";

/** `true` when running inside the Tauri window rather than a plain browser. */
export const isDesktop = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export class CwicoError extends Error {
  constructor(
    readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "CwicoError";
  }
}

function toCwicoError(error: unknown): CwicoError {
  if (error && typeof error === "object" && "code" in error && "message" in error) {
    const api = error as ApiError;
    return new CwicoError(api.code, api.message);
  }
  return new CwicoError("other", String(error));
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toCwicoError(error);
  }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

export async function about(): Promise<AboutInfo> {
  if (!isDesktop()) return fixtures.about();
  return call<AboutInfo>("about");
}

export async function scan(options?: ScanOptions): Promise<ScanReport> {
  if (!isDesktop()) return fixtures.scan(options);
  return call<ScanReport>("scan", { options: options ?? null });
}

export async function buildPlan(
  selections: Selection[],
  options?: PlanOptions,
): Promise<RemovalPlan> {
  if (!isDesktop()) return fixtures.buildPlan(selections, options);
  return call<RemovalPlan>("build_plan", {
    selections,
    options: options ?? null,
  });
}

export async function executePlan(plan: RemovalPlan): Promise<RunReport> {
  if (!isDesktop()) return fixtures.executePlan(plan);
  return call<RunReport>("execute_plan", { plan });
}

export async function tweakCatalog(): Promise<TweakCatalog> {
  if (!isDesktop()) return fixtures.tweakCatalog();
  return call<TweakCatalog>("tweak_catalog");
}

export async function applyTweaks(
  ids: string[],
  enable: boolean,
  dryRun: boolean,
): Promise<TweakOutcome[]> {
  if (!isDesktop()) return fixtures.applyTweaks(ids, enable);
  return call<TweakOutcome[]>("apply_tweaks", { ids, enable, dryRun });
}

/**
 * Open https://tsudev.com.
 *
 * The Rust command takes no URL: it always opens the product site. A command
 * that accepted an arbitrary URL would be a phishing primitive if the web
 * view were ever compromised.
 */
export async function openProductSite(): Promise<void> {
  if (!isDesktop()) {
    window.open("https://tsudev.com", "_blank", "noopener,noreferrer");
    return;
  }
  return call<void>("open_product_site");
}

export async function openBackupDir(): Promise<void> {
  if (!isDesktop()) return;
  return call<void>("open_backup_dir");
}

export async function relaunchAsAdmin(): Promise<void> {
  if (!isDesktop()) return;
  return call<void>("relaunch_as_admin");
}

// ---------------------------------------------------------------------------
// Updates
// ---------------------------------------------------------------------------

/**
 * Ask whether a newer release exists.
 *
 * Never throws: a check that could not run comes back with `checked: false`
 * and the app carries on. Failing open is deliberate - a server outage must
 * not lock every user out of the tool at once.
 */
export async function checkForUpdate(): Promise<UpdateStatus> {
  if (!isDesktop()) return fixtures.checkForUpdate();
  try {
    return await invoke<UpdateStatus>("check_for_update");
  } catch (error) {
    return {
      available: false,
      checked: false,
      checkError: String(error),
      currentRelease: "",
      currentVersion: "",
    };
  }
}

/** Download and install the update, then restart. Only returns on failure. */
export async function installUpdate(): Promise<void> {
  if (!isDesktop()) return fixtures.installUpdate();
  return call<void>("install_update");
}

export async function onUpdateProgress(
  handler: (progress: UpdateProgress) => void,
): Promise<UnlistenFn> {
  if (!isDesktop()) return fixtures.onUpdateProgress(handler);
  return listen<UpdateProgress>(UPDATE_PROGRESS_EVENT, (event) =>
    handler(event.payload),
  );
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------

/** Subscribe to engine progress. Returns an unsubscribe function. */
export async function onProgress(
  handler: (event: ProgressEvent) => void,
): Promise<UnlistenFn> {
  if (!isDesktop()) return fixtures.onProgress(handler);
  return listen<ProgressEvent>(PROGRESS_EVENT, (event) => handler(event.payload));
}
