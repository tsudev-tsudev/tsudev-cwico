/**
 * Mirrors the `serde` shapes exported by `cwico-core`.
 *
 * Kept hand-written rather than generated: the surface is small, and a
 * hand-written file is where a mismatch with the Rust side gets noticed.
 * Every Rust enum below is `#[serde(rename_all = "snake_case")]` or
 * `"lowercase"`, and every struct is `camelCase`.
 */

export type Locale = "vi" | "en";

export type SafetyClass = "safe" | "caution" | "unknown" | "critical";

export type SourceKind =
  | "registry_uninstall"
  | "appx_package"
  | "appx_provisioned"
  | "windows_service"
  | "scheduled_task"
  | "startup_entry"
  | "windows_capability"
  | "optional_feature"
  | "leftover";

export type InstallScope = "machine" | "user" | "allusers" | "unknown";

export type ItemState =
  | "running"
  | "stopped"
  | "enabled"
  | "disabled"
  | "installed"
  | "unknown";

export type Action =
  | "disable"
  | "enable"
  | "uninstall"
  | "uninstall_and_deep_clean"
  | "deep_clean_only";

export interface LocalizedText {
  en: string;
  vi: string;
}

export interface SoftwareItem {
  id: string;
  name: string;
  version?: string;
  publisher?: string;
  source: SourceKind;
  scope: InstallScope;
  arch: string;
  state: ItemState;
  safety: SafetyClass;
  safetyReason?: LocalizedText;
  description?: LocalizedText;
  installLocation?: string;
  uninstallString?: string;
  quietUninstallString?: string;
  sizeBytes?: number;
  installDate?: string;
  registryKey?: string;
  packageFullName?: string;
  packageFamilyName?: string;
  systemName?: string;
  executables: string[];
  canDisable: boolean;
  canUninstall: boolean;
  extra: Record<string, string>;
}

export interface ScanOptions {
  registryPrograms: boolean;
  appxPackages: boolean;
  appxProvisioned: boolean;
  services: boolean;
  scheduledTasks: boolean;
  startupEntries: boolean;
  optionalFeatures: boolean;
  leftovers: boolean;
  includeSystemComponents: boolean;
  includeNonRemovable: boolean;
  measureDiskUsage: boolean;
}

export interface ScanStats {
  total: number;
  byKind: Record<string, number>;
  bySafety: Record<string, number>;
  reclaimableBytes: number;
  durationMs: number;
}

export interface ScanWarning {
  pass: string;
  message: string;
}

export interface ScanReport {
  startedAt: string;
  options: ScanOptions;
  items: SoftwareItem[];
  warnings: ScanWarning[];
  stats: ScanStats;
  safetyDbVersion: string;
  elevated: boolean;
}

export interface Selection {
  itemId: string;
  action: Action;
  confirmed: boolean;
}

export interface PlanOptions {
  createRestorePoint: boolean;
  requireRestorePoint: boolean;
  backupRegistry: boolean;
  killProcesses: boolean;
  removeProvisioned: boolean;
  dryRun: boolean;
  continueOnError: boolean;
  backupDir?: string;
}

export interface PlanStep {
  kind: string;
  [key: string]: unknown;
}

export interface PlannedItem {
  itemId: string;
  name: string;
  source: SourceKind;
  safety: SafetyClass;
  action: Action;
  steps: PlanStep[];
  registryKeysAtRisk: string[];
}

export interface RejectedSelection {
  itemId: string;
  name: string;
  code: string;
  detail: string;
}

export interface RemovalPlan {
  options: PlanOptions;
  preamble: PlanStep[];
  items: PlannedItem[];
  rejected: RejectedSelection[];
}

export type StepStatus = "succeeded" | "skipped" | "failed" | "simulated";

export interface StepOutcome {
  step: string;
  status: StepStatus;
  detail: string;
  artifacts?: string[];
  errorCode?: string;
  durationMs: number;
  bytesFreed?: number;
}

export interface ItemOutcome {
  itemId: string;
  name: string;
  status: StepStatus;
  steps: StepOutcome[];
  bytesFreed: number;
}

export interface RestorePointInfo {
  sequenceNumber: number;
  description: string;
  createdAt: string;
}

export interface RegistryBackup {
  key: string;
  file: string;
  bytes: number;
}

export interface RunReport {
  startedAt: string;
  finishedAt: string;
  dryRun: boolean;
  restorePoint?: RestorePointInfo;
  registryBackups: RegistryBackup[];
  items: ItemOutcome[];
  succeeded: number;
  failed: number;
  skipped: number;
  bytesFreed: number;
  rebootRequired: boolean;
  transactionLog?: string;
  warnings: string[];
}

export interface PlatformInfo {
  platform: string;
  osDescription: string;
  osBuild?: string;
  arch: string;
  elevated: boolean;
  systemRestoreAvailable: boolean;
}

export interface AboutInfo {
  /** Raw semver, for support reports. */
  appVersion: string;
  /** The name users recognise: `tsudev-cwico-v26.8.19`. */
  appRelease: string;
  productUrl: string;
  platform: PlatformInfo;
  safetyDbVersion: string;
  safetyDbUpdated: string;
  safetyRules: number;
  safeRules: number;
  cautionRules: number;
  criticalRules: number;
  backupDir: string;
}

export type TweakCategory =
  | "privacy"
  | "performance"
  | "explorer"
  | "gaming"
  | "network"
  | "developer"
  | "interface"
  | "cleanup";

export interface Tweak {
  id: string;
  category: TweakCategory;
  title: LocalizedText;
  description: LocalizedText;
  safety: SafetyClass;
  requiresRestart?: boolean;
  apply: unknown[];
  revert: unknown[];
  tags?: string[];
  recommended?: boolean;
}

export interface TweakCatalog {
  schemaVersion: number;
  version: string;
  tweaks: Tweak[];
}

export interface TweakOutcome {
  id: string;
  ok: boolean;
  detail: string;
}

/** Progress events streamed on `cwico://progress`. */
export type ProgressEvent =
  | { type: "scanStarted"; passes: number }
  | { type: "scanPassStarted"; pass: string; index: number; total: number }
  | { type: "scanPassFinished"; pass: string; found: number }
  | { type: "scanFinished"; total: number; durationMs: number }
  | { type: "runStarted"; totalSteps: number; dryRun: boolean }
  | {
      type: "stepStarted";
      itemId?: string;
      step: string;
      index: number;
      total: number;
    }
  | {
      type: "stepFinished";
      itemId?: string;
      step: string;
      status: StepStatus;
      detail: string;
    }
  | { type: "itemFinished"; itemId: string; name: string; status: StepStatus }
  | {
      type: "runFinished";
      succeeded: number;
      failed: number;
      skipped: number;
      durationMs: number;
    }
  | { type: "log"; level: "debug" | "info" | "warn" | "error"; message: string };

export interface ApiError {
  code: string;
  message: string;
}

export const DEFAULT_SCAN_OPTIONS: ScanOptions = {
  registryPrograms: true,
  appxPackages: true,
  appxProvisioned: true,
  services: true,
  scheduledTasks: true,
  startupEntries: true,
  optionalFeatures: true,
  leftovers: false,
  includeSystemComponents: false,
  includeNonRemovable: false,
  measureDiskUsage: false,
};

export const DEFAULT_PLAN_OPTIONS: PlanOptions = {
  createRestorePoint: true,
  requireRestorePoint: true,
  backupRegistry: true,
  killProcesses: true,
  removeProvisioned: true,
  dryRun: false,
  continueOnError: true,
};

/**
 * The result of an update check.
 *
 * `available` is the only field that closes the update gate. `checked: false`
 * means the check could not run - offline, DNS, GitHub down - and the app
 * carries on normally; see `app/src-tauri/src/update.rs` for why that
 * distinction matters.
 */
export interface UpdateStatus {
  available: boolean;
  checked: boolean;
  checkError?: string;
  currentRelease: string;
  currentVersion: string;
  newRelease?: string;
  newVersion?: string;
  notes?: string;
  publishedAt?: string;
}

export interface UpdateProgress {
  downloaded: number;
  total?: number;
  installing: boolean;
}
