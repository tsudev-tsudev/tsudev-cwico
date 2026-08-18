/**
 * Live progress, and the report at the end.
 *
 * Progress is shown per step rather than as a single bar, because "removing
 * OneDrive: terminating processes" is the kind of detail that makes a
 * long-running destructive operation feel accountable rather than opaque.
 */

import { useEffect, useRef } from "react";
import type { Locale, ProgressEvent, RunReport } from "../types";
import { formatBytes, translator } from "../i18n";
import { Button, Modal, Notice, ProgressBar } from "./primitives";
import { openBackupDir } from "../api";

export interface LogLine {
  id: number;
  text: string;
  status?: "succeeded" | "skipped" | "failed" | "simulated";
}

export function eventToLine(event: ProgressEvent, id: number): LogLine | null {
  switch (event.type) {
    case "scanPassStarted":
      return { id, text: `[${event.index}/${event.total}] ${event.pass}…` };
    case "scanPassFinished":
      return { id, text: `${event.pass}: ${event.found}`, status: "succeeded" };
    case "stepStarted":
      return { id, text: `[${event.index}/${event.total}] ${event.step}…` };
    case "stepFinished":
      return { id, text: `${event.step} — ${event.detail}`, status: event.status };
    case "itemFinished":
      return { id, text: `→ ${event.name}`, status: event.status };
    case "log":
      return { id, text: `${event.level}: ${event.message}` };
    default:
      return null;
  }
}

interface Props {
  open: boolean;
  locale: Locale;
  running: boolean;
  progress?: number;
  lines: LogLine[];
  report: RunReport | null;
  onClose: () => void;
}

export function RunPanel({
  open,
  locale,
  running,
  progress,
  lines,
  report,
  onClose,
}: Props) {
  const t = translator(locale);
  const bottom = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [lines.length]);

  const statusColour = (status?: LogLine["status"]) => {
    switch (status) {
      case "failed":
        return "#b3261e";
      case "skipped":
        return "var(--text-muted)";
      case "simulated":
        return "var(--color-dev-500)";
      case "succeeded":
        return "var(--color-tsu-500)";
      default:
        return "var(--text-secondary)";
    }
  };

  return (
    <Modal
      open={open}
      title={running ? t("run.title") : t("run.done")}
      onClose={running ? () => {} : onClose}
      footer={
        <>
          {report && !report.dryRun && (
            <Button variant="ghost" onClick={() => void openBackupDir()}>
              {t("action.openBackups")}
            </Button>
          )}
          <Button variant="primary" onClick={onClose} disabled={running}>
            {t("action.close")}
          </Button>
        </>
      }
    >
      {running && (
        <div className="mb-4">
          <ProgressBar value={progress} />
        </div>
      )}

      {report && (
        <div className="mb-4 space-y-3">
          {report.dryRun && <Notice tone="info" title={t("plan.dryRun")} />}
          {report.failed > 0 && (
            <Notice tone="danger" title={`${report.failed} ${t("run.failed")}`} />
          )}
          {report.failed === 0 && !report.dryRun && (
            <Notice
              tone="success"
              title={`${report.succeeded} ${t("run.succeeded")} · ${formatBytes(
                locale,
                report.bytesFreed,
              )} ${t("run.freed")}`}
            />
          )}
          {report.restorePoint && (
            <p className="text-[13px]" style={{ color: "var(--text-secondary)" }}>
              {t("run.restorePointCreated")} #{report.restorePoint.sequenceNumber}
            </p>
          )}
          {report.transactionLog && (
            <p className="selectable break-all font-mono text-[12px]" style={{ color: "var(--text-muted)" }}>
              {t("run.transactionLog")}: {report.transactionLog}
            </p>
          )}
          {report.rebootRequired && (
            <Notice tone="warning" title={t("run.rebootRequired")} />
          )}
          {report.warnings.map((warning) => (
            <Notice key={warning} tone="warning" title={warning} />
          ))}
        </div>
      )}

      <div
        className="selectable max-h-72 overflow-y-auto rounded-lg border p-3 font-mono text-[12px] leading-relaxed"
        style={{
          background: "var(--surface-sunken)",
          borderColor: "var(--border-subtle)",
        }}
      >
        {lines.map((line) => (
          <div key={line.id} style={{ color: statusColour(line.status) }}>
            {line.text}
          </div>
        ))}
        <div ref={bottom} />
      </div>
    </Modal>
  );
}
