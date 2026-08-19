/**
 * The blocking update screen.
 *
 * When a newer release is confirmed, this replaces the entire application.
 * There is no dismiss, no "later", and no way past it except installing —
 * which is the point.
 *
 * ## Why blocking, for a tool that usually should not
 *
 * The safety database decides what this tool will and will not remove. A
 * correction to it — something classified `Safe` that turned out to cost a
 * user a feature, or something that should have been `Critical` — ships as a
 * new version. A user on last month's build is running last month's idea of
 * what is safe to delete, with Administrator rights.
 *
 * The gate is nonetheless only reached when a newer version is *confirmed*.
 * A failed check leaves the app running normally; see `update.rs`.
 */

import { useEffect, useState } from "react";
import type { Locale, UpdateProgress, UpdateStatus } from "../types";
import { formatBytes, formatDateTime, translator } from "../i18n";
import * as api from "../api";
import { Brand } from "./Brand";
import { Button, Notice, ProgressBar } from "./primitives";
import { parseNotes } from "../releaseNotes";

type Phase = "idle" | "downloading" | "installing" | "failed";

export function UpdateGate({
  status,
  locale,
  onLocaleToggle,
}: {
  status: UpdateStatus;
  locale: Locale;
  onLocaleToggle: () => void;
}) {
  const t = translator(locale);
  const [phase, setPhase] = useState<Phase>("idle");
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void api
      .onUpdateProgress((p) => {
        setProgress(p);
        setPhase(p.installing ? "installing" : "downloading");
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  }, []);

  const install = async () => {
    setError(null);
    setPhase("downloading");
    try {
      await api.installUpdate();
      // Only reached if the installer returned without replacing the process.
      setPhase("installing");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setPhase("failed");
    }
  };

  const busy = phase === "downloading" || phase === "installing";
  const fraction =
    progress?.total && progress.total > 0
      ? progress.downloaded / progress.total
      : undefined;

  const phaseLabel = () => {
    if (phase === "installing") return t("update.installing");
    if (phase === "downloading") return t("update.downloading");
    return null;
  };

  return (
    <div
      className="flex h-full flex-col items-center justify-center px-6"
      style={{ background: "var(--surface-app)" }}
    >
      <div className="w-full max-w-lg">
        <div className="mb-7 flex items-center justify-between">
          <Brand size="md" />
          <button
            type="button"
            onClick={onLocaleToggle}
            title={t("locale.toggle")}
            className="rounded-lg border px-2.5 py-1.5 text-xs font-semibold uppercase"
            style={{
              borderColor: "var(--border-strong)",
              color: "var(--text-secondary)",
            }}
          >
            {locale === "vi" ? "VI" : "EN"}
          </button>
        </div>

        <div
          className="rounded-xl border p-6"
          style={{
            background: "var(--surface-panel)",
            borderColor: "var(--border-subtle)",
            boxShadow: "var(--shadow-panel)",
          }}
        >
          <h1 className="text-lg font-semibold">{t("update.gate.title")}</h1>
          <p className="mt-1.5 text-sm" style={{ color: "var(--text-secondary)" }}>
            {t("update.gate.lead")}
          </p>

          <dl
            className="mt-5 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 rounded-lg px-4 py-3 text-[13px]"
            style={{ background: "var(--surface-sunken)" }}
          >
            <dt style={{ color: "var(--text-muted)" }}>{t("update.current")}</dt>
            <dd className="selectable font-mono">{status.currentRelease}</dd>
            <dt style={{ color: "var(--text-muted)" }}>{t("update.new")}</dt>
            <dd
              className="selectable font-mono font-semibold"
              style={{ color: "var(--color-dev-600)" }}
            >
              {status.newRelease ?? status.newVersion}
            </dd>
            {status.publishedAt && (
              <>
                <dt style={{ color: "var(--text-muted)" }}>
                  {t("update.published")}
                </dt>
                <dd>{formatDateTime(locale, status.publishedAt)}</dd>
              </>
            )}
          </dl>

          {status.notes && (
            <div className="mt-4">
              <p
                className="mb-1 text-[11px] font-semibold uppercase tracking-wide"
                style={{ color: "var(--text-muted)" }}
              >
                {t("update.notes")}
              </p>
              <div
                className="selectable max-h-40 space-y-2 overflow-y-auto pr-1 text-[13px]"
                style={{ color: "var(--text-secondary)" }}
              >
                {parseNotes(status.notes).map((block, index) => {
                  if (block.kind === "code") {
                    return (
                      <pre
                        key={index}
                        className="overflow-x-auto rounded px-2.5 py-2 font-mono text-[12px]"
                        style={{ background: "var(--surface-sunken)" }}
                      >
                        {block.text}
                      </pre>
                    );
                  }
                  if (block.kind === "bullet") {
                    return (
                      <p key={index} className="flex gap-2">
                        <span aria-hidden="true" style={{ color: "var(--text-muted)" }}>
                          •
                        </span>
                        <span>{block.text}</span>
                      </p>
                    );
                  }
                  return <p key={index}>{block.text}</p>;
                })}
              </div>
            </div>
          )}

          {busy && (
            <div className="mt-5">
              <div className="mb-1.5 flex items-baseline justify-between text-[12px]">
                <span style={{ color: "var(--text-secondary)" }}>{phaseLabel()}</span>
                {phase === "downloading" && progress && (
                  <span className="tabular-nums" style={{ color: "var(--text-muted)" }}>
                    {formatBytes(locale, progress.downloaded)}
                    {progress.total ? ` / ${formatBytes(locale, progress.total)}` : ""}
                  </span>
                )}
              </div>
              <ProgressBar value={phase === "installing" ? undefined : fraction} />
              {phase === "installing" && (
                <p className="mt-2 text-[12px]" style={{ color: "var(--text-muted)" }}>
                  {t("update.restarting")}
                </p>
              )}
            </div>
          )}

          {phase === "failed" && error && (
            <div className="mt-5">
              <Notice tone="danger" title={t("update.failed")}>
                <span className="selectable break-all font-mono text-[12px]">
                  {error}
                </span>
              </Notice>
            </div>
          )}

          <div className="mt-6">
            <Button
              variant="primary"
              onClick={() => void install()}
              disabled={busy}
              className="w-full justify-center py-2.5 text-base"
            >
              {phase === "failed" ? t("update.retry") : t("update.button")}
            </Button>
          </div>

          <p
            className="mt-5 border-t pt-4 text-[12px] leading-relaxed"
            style={{
              borderColor: "var(--border-subtle)",
              color: "var(--text-muted)",
            }}
          >
            {t("update.gate.why")}
          </p>
        </div>
      </div>
    </div>
  );
}
