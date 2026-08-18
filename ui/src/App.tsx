/**
 * Application shell and the one place selection state lives.
 *
 * The flow the whole interface is built around:
 *
 *   scan → select → build a plan → read what was refused → confirm → run
 *
 * The plan step is not optional and not skippable. Every destructive action
 * passes through `buildPlan`, which is the same Rust code path the CLI uses,
 * so the front end cannot invent a shortcut around the safety gate.
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import * as api from "./api";
import { CwicoError } from "./api";
import { formatDateTime, translator } from "./i18n";
import type {
  AboutInfo,
  Locale,
  ProgressEvent,
  RemovalPlan,
  RunReport,
  ScanOptions,
  ScanReport,
  Selection,
  SoftwareItem,
  Tweak,
  TweakOutcome,
} from "./types";
import { DEFAULT_PLAN_OPTIONS, DEFAULT_SCAN_OPTIONS } from "./types";
import { Brand } from "./components/Brand";
import { AboutView } from "./components/AboutView";
import { ConfirmCaution } from "./components/ConfirmCaution";
import { PlanDialog } from "./components/PlanDialog";
import { RunPanel, eventToLine, type LogLine } from "./components/RunPanel";
import { SoftwareTable } from "./components/SoftwareTable";
import { TweaksView } from "./components/TweaksView";
import { Button, Notice, Panel, ProgressBar } from "./components/primitives";
import { formatBytes } from "./i18n";

type Tab = "software" | "tweaks" | "activity" | "about";

const THEME_KEY = "cwico.theme";
const LOCALE_KEY = "cwico.locale";

export default function App() {
  const [locale, setLocale] = useState<Locale>(
    () => (localStorage.getItem(LOCALE_KEY) as Locale) ?? "vi",
  );
  const [dark, setDark] = useState(
    () =>
      localStorage.getItem(THEME_KEY) === "dark" ||
      (localStorage.getItem(THEME_KEY) === null &&
        window.matchMedia("(prefers-color-scheme: dark)").matches),
  );
  const [tab, setTab] = useState<Tab>("software");

  const [about, setAbout] = useState<AboutInfo | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [scanning, setScanning] = useState(false);
  const [deepScan, setDeepScan] = useState(false);

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [confirmed, setConfirmed] = useState<Set<string>>(new Set());
  const [deepClean, setDeepClean] = useState(true);

  /** The Caution/Unknown item awaiting its own acknowledgement. */
  const [pendingConfirm, setPendingConfirm] = useState<SoftwareItem | null>(null);

  const [plan, setPlan] = useState<RemovalPlan | null>(null);
  const [planOpen, setPlanOpen] = useState(false);

  const [running, setRunning] = useState(false);
  const [runOpen, setRunOpen] = useState(false);
  const [runReport, setRunReport] = useState<RunReport | null>(null);
  const [progress, setProgress] = useState<number | undefined>(undefined);

  const [lines, setLines] = useState<LogLine[]>([]);
  const [error, setError] = useState<string | null>(null);

  const [tweaks, setTweaks] = useState<Tweak[]>([]);
  const [tweakBusy, setTweakBusy] = useState(false);
  const [tweakOutcomes, setTweakOutcomes] = useState<TweakOutcome[]>([]);

  const t = translator(locale);

  /* ------------------------------------------------------------- effects */

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem(THEME_KEY, dark ? "dark" : "light");
  }, [dark]);

  useEffect(() => {
    localStorage.setItem(LOCALE_KEY, locale);
    document.documentElement.lang = locale;
  }, [locale]);

  useEffect(() => {
    api.about().then(setAbout).catch(() => setAbout(null));
    api
      .tweakCatalog()
      .then((catalog) => setTweaks(catalog.tweaks))
      .catch(() => setTweaks([]));
  }, []);

  useEffect(() => {
    let counter = 0;
    let unlisten: (() => void) | undefined;

    void api
      .onProgress((event: ProgressEvent) => {
        const line = eventToLine(event, (counter += 1));
        if (line) setLines((previous) => [...previous.slice(-400), line]);

        if (event.type === "stepStarted") {
          setProgress(event.index / Math.max(event.total, 1));
        }
        if (event.type === "runFinished" || event.type === "scanFinished") {
          setProgress(undefined);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => unlisten?.();
  }, []);

  /* -------------------------------------------------------------- actions */

  const handleError = useCallback((cause: unknown) => {
    setError(
      cause instanceof CwicoError
        ? `${cause.message} (${cause.code})`
        : String(cause),
    );
  }, []);

  const runScan = useCallback(async () => {
    setScanning(true);
    setError(null);
    setLines([]);
    try {
      const options: ScanOptions = deepScan
        ? {
            ...DEFAULT_SCAN_OPTIONS,
            leftovers: true,
            includeSystemComponents: true,
            includeNonRemovable: true,
            measureDiskUsage: true,
          }
        : DEFAULT_SCAN_OPTIONS;
      const result = await api.scan(options);
      setReport(result);
      // Ids are stable across scans, but an item that vanished must not stay
      // selected or the next plan would fail on a ghost.
      const present = new Set(result.items.map((item) => item.id));
      setSelected((previous) => new Set([...previous].filter((id) => present.has(id))));
    } catch (cause) {
      handleError(cause);
    } finally {
      setScanning(false);
    }
  }, [deepScan, handleError]);

  const items = report?.items ?? [];

  const deselect = useCallback((id: string) => {
    setSelected((previous) => {
      const next = new Set(previous);
      next.delete(id);
      return next;
    });
    setConfirmed((previous) => {
      const next = new Set(previous);
      next.delete(id);
      return next;
    });
  }, []);

  const toggleItem = useCallback(
    (item: SoftwareItem) => {
      // Critical items are not selectable at all — the row renders a lock
      // instead of a checkbox, and this is the second line of that defence.
      if (item.safety === "critical") return;

      if (selected.has(item.id)) {
        deselect(item.id);
        return;
      }

      // Caution and Unknown items need a deliberate acknowledgement, not a
      // tick. The dialog is what sets `confirmed`, and `confirmed` is what
      // the engine requires before it will plan the removal.
      if (item.safety === "caution" || item.safety === "unknown") {
        setPendingConfirm(item);
        return;
      }

      setSelected((previous) => new Set(previous).add(item.id));
    },
    [selected, deselect],
  );

  const acceptPendingConfirm = useCallback(() => {
    if (!pendingConfirm) return;
    setSelected((previous) => new Set(previous).add(pendingConfirm.id));
    setConfirmed((previous) => new Set(previous).add(pendingConfirm.id));
    setPendingConfirm(null);
  }, [pendingConfirm]);

  const selectAllSafe = useCallback(() => {
    setSelected(
      new Set(items.filter((item) => item.safety === "safe").map((item) => item.id)),
    );
    setConfirmed(new Set());
  }, [items]);

  const clearSelection = useCallback(() => {
    setSelected(new Set());
    setConfirmed(new Set());
  }, []);

  const selections = useMemo<Selection[]>(
    () =>
      [...selected].map((id) => ({
        itemId: id,
        action: deepClean ? "uninstall_and_deep_clean" : "uninstall",
        confirmed: confirmed.has(id),
      })),
    [selected, confirmed, deepClean],
  );

  const reviewPlan = useCallback(async () => {
    setError(null);
    try {
      const built = await api.buildPlan(selections, {
        ...DEFAULT_PLAN_OPTIONS,
        dryRun: false,
      });
      setPlan(built);
      setPlanOpen(true);
    } catch (cause) {
      handleError(cause);
    }
  }, [selections, handleError]);

  const execute = useCallback(
    async (dryRun: boolean) => {
      if (!plan) return;
      setPlanOpen(false);
      setLines([]);
      setRunReport(null);
      setRunOpen(true);
      setRunning(true);
      try {
        const target: RemovalPlan = dryRun
          ? await api.buildPlan(selections, { ...DEFAULT_PLAN_OPTIONS, dryRun: true })
          : plan;
        const result = await api.executePlan(target);
        setRunReport(result);
        if (!dryRun) {
          clearSelection();
          await runScan();
        }
      } catch (cause) {
        handleError(cause);
      } finally {
        setRunning(false);
        setProgress(undefined);
      }
    },
    [plan, selections, clearSelection, runScan, handleError],
  );

  const applyTweaks = useCallback(
    async (ids: string[], enable: boolean) => {
      setTweakBusy(true);
      setError(null);
      try {
        setTweakOutcomes(await api.applyTweaks(ids, enable, false));
      } catch (cause) {
        handleError(cause);
      } finally {
        setTweakBusy(false);
      }
    },
    [handleError],
  );

  /* ---------------------------------------------------------------- view */

  const elevated = about?.platform.elevated ?? false;
  const selectedItems = items.filter((item) => selected.has(item.id));
  const selectedBytes = selectedItems.reduce(
    (sum, item) => sum + (item.sizeBytes ?? 0),
    0,
  );

  return (
    <div className="flex h-full flex-col">
      {/* --------------------------------------------------------- header */}
      <header
        className="flex shrink-0 items-center gap-4 border-b px-5 py-3"
        style={{
          background: "var(--surface-panel)",
          borderColor: "var(--border-subtle)",
        }}
      >
        <Brand size="md" tagline={t("app.subtitle")} />

        <nav className="ml-4 flex items-center gap-1">
          {(
            [
              ["software", t("nav.software")],
              ["tweaks", t("nav.tweaks")],
              ["activity", t("nav.activity")],
              ["about", t("nav.about")],
            ] as [Tab, string][]
          ).map(([id, label]) => (
            <button
              key={id}
              type="button"
              onClick={() => setTab(id)}
              className="rounded-lg px-3 py-1.5 text-sm font-medium transition-colors"
              style={
                tab === id
                  ? {
                      background:
                        "color-mix(in srgb, var(--color-tsu-500) 14%, transparent)",
                      color: "var(--color-tsu-600)",
                    }
                  : { color: "var(--text-secondary)" }
              }
            >
              {label}
            </button>
          ))}
        </nav>

        <span className="flex-1" />

        <button
          type="button"
          onClick={() => setLocale(locale === "vi" ? "en" : "vi")}
          title={t("locale.toggle")}
          className="rounded-lg border px-2.5 py-1.5 text-xs font-semibold uppercase"
          style={{ borderColor: "var(--border-strong)", color: "var(--text-secondary)" }}
        >
          {locale === "vi" ? "VI" : "EN"}
        </button>
        <button
          type="button"
          onClick={() => setDark(!dark)}
          title={t("theme.toggle")}
          aria-label={t("theme.toggle")}
          className="rounded-lg border px-2.5 py-1.5 text-sm"
          style={{ borderColor: "var(--border-strong)" }}
        >
          {dark ? "☀" : "☾"}
        </button>
      </header>

      {/* ----------------------------------------------------------- body */}
      <main className="flex min-h-0 flex-1 flex-col gap-3 px-5 py-4">
        {error && (
          <Notice
            tone="danger"
            title={t("error.title")}
            action={
              <Button variant="ghost" onClick={() => setError(null)}>
                {t("action.close")}
              </Button>
            }
          >
            <span className="selectable">{error}</span>
          </Notice>
        )}

        {about && !elevated && (
          <Notice
            tone="warning"
            title={t("elevation.required")}
            action={
              <Button variant="outline" onClick={() => void api.relaunchAsAdmin()}>
                {t("elevation.relaunch")}
              </Button>
            }
          >
            {t("elevation.body")}
          </Notice>
        )}

        {about && elevated && !about.platform.systemRestoreAvailable && (
          <Notice tone="warning" title={t("restore.unavailable")}>
            {t("restore.unavailableBody")}
          </Notice>
        )}

        {tab === "software" && (
          <>
            <div className="flex flex-wrap items-center gap-3">
              <Button variant="primary" onClick={() => void runScan()} disabled={scanning}>
                {scanning ? t("scan.running") : report ? t("scan.rerun") : t("scan.run")}
              </Button>
              <label
                className="flex items-center gap-2 text-[13px]"
                style={{ color: "var(--text-secondary)" }}
                title={t("scan.deepHint")}
              >
                <input
                  type="checkbox"
                  checked={deepScan}
                  onChange={(event) => setDeepScan(event.target.checked)}
                  className="h-4 w-4 accent-[var(--color-tsu-600)]"
                />
                {t("scan.deep")}
              </label>
              <label
                className="flex items-center gap-2 text-[13px]"
                style={{ color: "var(--text-secondary)" }}
              >
                <input
                  type="checkbox"
                  checked={deepClean}
                  onChange={(event) => setDeepClean(event.target.checked)}
                  className="h-4 w-4 accent-[var(--color-dev-500)]"
                />
                {t("plan.deepClean")}
              </label>

              {report && (
                <span className="text-[13px]" style={{ color: "var(--text-muted)" }}>
                  {t("scan.lastScan")}: {formatDateTime(locale, report.startedAt)} ·{" "}
                  {t("scan.reclaimable")}: {formatBytes(locale, report.stats.reclaimableBytes)}
                </span>
              )}
            </div>

            {scanning && <ProgressBar />}

            {!report && !scanning ? (
              <Panel className="flex flex-1 flex-col items-center justify-center gap-4 text-center">
                <Brand size="lg" />
                <p className="max-w-md text-sm" style={{ color: "var(--text-secondary)" }}>
                  {t("scan.empty")}
                </p>
                <Button variant="primary" onClick={() => void runScan()}>
                  {t("scan.run")}
                </Button>
              </Panel>
            ) : (
              report && (
                <SoftwareTable
                  items={items}
                  locale={locale}
                  selected={selected}
                  confirmed={confirmed}
                  onToggle={toggleItem}
                  onSelectAllSafe={selectAllSafe}
                  onClearSelection={clearSelection}
                />
              )
            )}
          </>
        )}

        {tab === "tweaks" && (
          <TweaksView
            tweaks={tweaks}
            locale={locale}
            busy={tweakBusy}
            outcomes={tweakOutcomes}
            onApply={(ids, enable) => void applyTweaks(ids, enable)}
          />
        )}

        {tab === "activity" && (
          <Panel className="min-h-0 flex-1 overflow-y-auto">
            <h2 className="mb-3 text-sm font-semibold">{t("activity.title")}</h2>
            {lines.length === 0 ? (
              <p className="text-sm" style={{ color: "var(--text-muted)" }}>
                {t("activity.empty")}
              </p>
            ) : (
              <div className="selectable font-mono text-[12px] leading-relaxed">
                {lines.map((line) => (
                  <div key={line.id}>{line.text}</div>
                ))}
              </div>
            )}
          </Panel>
        )}

        {tab === "about" && <AboutView info={about} locale={locale} />}
      </main>

      {/* ---------------------------------------------------- action bar */}
      {tab === "software" && selected.size > 0 && (
        <div
          className="flex shrink-0 items-center gap-4 border-t px-5 py-3"
          style={{
            background: "var(--surface-panel)",
            borderColor: "var(--border-subtle)",
          }}
        >
          <span className="text-sm">
            <strong className="tabular-nums">{selected.size}</strong> {t("select.count")}
            {selectedBytes > 0 && (
              <span style={{ color: "var(--text-muted)" }}>
                {" "}
                · {formatBytes(locale, selectedBytes)}
              </span>
            )}
          </span>
          <span className="flex-1" />
          <Button variant="ghost" onClick={clearSelection}>
            {t("select.none")}
          </Button>
          <Button variant="primary" onClick={() => void reviewPlan()}>
            {t("action.review")} →
          </Button>
        </div>
      )}

      {/* --------------------------------------------------------- footer */}
      <footer
        className="flex shrink-0 items-center justify-between gap-3 border-t px-5 py-2 text-[11px]"
        style={{
          background: "var(--surface-sunken)",
          borderColor: "var(--border-subtle)",
          color: "var(--text-muted)",
        }}
      >
        <Brand size="sm" />
        <span>
          v{about?.appVersion ?? "1.0.0"}
          {about && ` · ${t("about.safetyDb")} v${about.safetyDbVersion}`}
        </span>
      </footer>

      <ConfirmCaution
        item={pendingConfirm}
        locale={locale}
        onConfirm={acceptPendingConfirm}
        onCancel={() => setPendingConfirm(null)}
      />

      <PlanDialog
        open={planOpen}
        plan={plan}
        locale={locale}
        busy={running}
        onClose={() => setPlanOpen(false)}
        onDryRun={() => void execute(true)}
        onExecute={() => void execute(false)}
      />

      <RunPanel
        open={runOpen}
        locale={locale}
        running={running}
        progress={progress}
        lines={lines}
        report={runReport}
        onClose={() => setRunOpen(false)}
      />
    </div>
  );
}
