/**
 * The about panel.
 *
 * Beyond version numbers, it reports how much protection is actually loaded —
 * how many Safe, Caution and Critical rules the safety database contains — so
 * a user can tell at a glance whether the guard rails are in place, and where
 * the rollback artefacts for their last run were written.
 */

import type { AboutInfo, Locale } from "../types";
import { translator } from "../i18n";
import { Brand } from "./Brand";
import { Button, Notice, Panel } from "./primitives";
import { openBackupDir, openProductSite } from "../api";

export function AboutView({
  info,
  locale,
}: {
  info: AboutInfo | null;
  locale: Locale;
}) {
  const t = translator(locale);
  if (!info) return null;

  const rows: [string, string][] = [
    [t("about.engine"), `cwico-core ${info.appVersion}`],
    ["Release", info.appRelease],
    [t("about.os"), info.platform.osDescription],
    ["Architecture", info.platform.arch],
    [
      t("about.safetyDb"),
      `v${info.safetyDbVersion} (${info.safetyDbUpdated}) — ${info.safetyRules} ${t(
        "about.rules",
      )}`,
    ],
    [t("about.backupDir"), info.backupDir],
  ];

  return (
    <div className="mx-auto flex max-w-2xl flex-col gap-4">
      <Panel className="flex flex-col items-center gap-4 py-8 text-center">
        <Brand size="lg" />
        <div>
          <p className="text-sm font-medium">{t("app.subtitle")}</p>
          <p className="selectable mt-1 text-[13px]" style={{ color: "var(--text-muted)" }}>
            {info.appRelease} · MIT
          </p>
        </div>
        <Button variant="primary" onClick={() => void openProductSite()}>
          {t("about.website")} ↗
        </Button>
      </Panel>

      <Panel>
        <h3 className="mb-3 text-sm font-semibold">{t("about.safetyDb")}</h3>
        <div className="grid grid-cols-3 gap-3">
          {[
            { label: t("safety.safe"), value: info.safeRules, colour: "var(--color-tsu-500)" },
            { label: t("safety.caution"), value: info.cautionRules, colour: "var(--color-dev-500)" },
            { label: t("safety.critical"), value: info.criticalRules, colour: "#b3261e" },
          ].map((cell) => (
            <div
              key={cell.label}
              className="rounded-lg border px-3 py-3 text-center"
              style={{
                borderColor: `color-mix(in srgb, ${cell.colour} 30%, transparent)`,
                background: `color-mix(in srgb, ${cell.colour} 7%, transparent)`,
              }}
            >
              <p className="text-2xl font-semibold tabular-nums" style={{ color: cell.colour }}>
                {cell.value}
              </p>
              <p className="text-[12px]" style={{ color: "var(--text-secondary)" }}>
                {cell.label}
              </p>
            </div>
          ))}
        </div>
      </Panel>

      <Panel>
        <dl className="space-y-2.5">
          {rows.map(([label, value]) => (
            <div key={label} className="flex flex-wrap items-baseline gap-2">
              <dt
                className="w-40 shrink-0 text-[12px] font-semibold uppercase tracking-wide"
                style={{ color: "var(--text-muted)" }}
              >
                {label}
              </dt>
              <dd className="selectable min-w-0 flex-1 break-all font-mono text-[12px]">
                {value}
              </dd>
            </div>
          ))}
        </dl>
        <div className="mt-4">
          <Button variant="outline" onClick={() => void openBackupDir()}>
            {t("action.openBackups")}
          </Button>
        </div>
      </Panel>

      {!info.platform.systemRestoreAvailable && (
        <Notice tone="warning" title={t("restore.unavailable")}>
          {t("restore.unavailableBody")}
        </Notice>
      )}
    </div>
  );
}
