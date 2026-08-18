/**
 * The tweaks catalogue.
 *
 * This is what the original `Optimize_Win11_For_Dev.ps1` became. That script
 * applied twelve fixed groups of changes with no way to inspect, choose or
 * undo them; here every change is a row with its own safety class, its own
 * explanation of what it costs, and a revert path where one exists. Tweaks
 * that cannot be undone say so on the row rather than in a README.
 */

import { useMemo, useState } from "react";
import type { Locale, Tweak, TweakCategory, TweakOutcome } from "../types";
import { translator } from "../i18n";
import { Button, Notice, Panel, SafetyBadge, Tag } from "./primitives";

const CATEGORY_LABEL: Record<TweakCategory, { vi: string; en: string }> = {
  privacy: { vi: "Quyền riêng tư", en: "Privacy" },
  performance: { vi: "Hiệu năng", en: "Performance" },
  explorer: { vi: "File Explorer", en: "File Explorer" },
  gaming: { vi: "Chơi game", en: "Gaming" },
  network: { vi: "Mạng", en: "Network" },
  developer: { vi: "Lập trình", en: "Developer" },
  interface: { vi: "Giao diện", en: "Interface" },
  cleanup: { vi: "Dọn dẹp", en: "Cleanup" },
};

const ORDER: TweakCategory[] = [
  "privacy",
  "performance",
  "explorer",
  "gaming",
  "developer",
  "network",
  "interface",
  "cleanup",
];

interface Props {
  tweaks: Tweak[];
  locale: Locale;
  busy: boolean;
  outcomes: TweakOutcome[];
  onApply: (ids: string[], enable: boolean) => void;
}

export function TweaksView({ tweaks, locale, busy, outcomes, onApply }: Props) {
  const t = translator(locale);
  const [selected, setSelected] = useState<Set<string>>(new Set());

  const grouped = useMemo(() => {
    const map = new Map<TweakCategory, Tweak[]>();
    for (const category of ORDER) map.set(category, []);
    for (const tweak of tweaks) {
      map.get(tweak.category)?.push(tweak);
    }
    return [...map.entries()].filter(([, list]) => list.length > 0);
  }, [tweaks]);

  const toggle = (id: string) => {
    setSelected((previous) => {
      const next = new Set(previous);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const selectRecommended = () => {
    setSelected(new Set(tweaks.filter((tweak) => tweak.recommended).map((t) => t.id)));
  };

  const outcomeFor = (id: string) => outcomes.find((outcome) => outcome.id === id);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <Button variant="ghost" onClick={selectRecommended} disabled={busy}>
          {t("tweaks.recommended")}
        </Button>
        <Button variant="ghost" onClick={() => setSelected(new Set())} disabled={!selected.size}>
          {t("select.none")}
        </Button>
        <span className="flex-1" />
        <Button
          variant="outline"
          onClick={() => onApply([...selected], false)}
          disabled={busy || !selected.size}
        >
          {t("tweaks.revert")}
        </Button>
        <Button
          variant="primary"
          onClick={() => onApply([...selected], true)}
          disabled={busy || !selected.size}
        >
          {t("tweaks.apply")} ({selected.size})
        </Button>
      </div>

      <p className="text-[13px]" style={{ color: "var(--text-muted)" }}>
        {t("tweaks.subtitle")}
      </p>

      <div className="min-h-0 flex-1 space-y-5 overflow-y-auto pr-1">
        {grouped.map(([category, list]) => (
          <section key={category}>
            <h3
              className="mb-2 text-[11px] font-semibold uppercase tracking-wide"
              style={{ color: "var(--text-muted)" }}
            >
              {CATEGORY_LABEL[category][locale]}
            </h3>
            <Panel padded={false}>
              <ul>
                {list.map((tweak) => {
                  const outcome = outcomeFor(tweak.id);
                  return (
                    <li
                      key={tweak.id}
                      className="flex items-start gap-3 border-b px-3.5 py-2.5 last:border-b-0"
                      style={{ borderColor: "var(--border-subtle)" }}
                    >
                      <input
                        type="checkbox"
                        checked={selected.has(tweak.id)}
                        onChange={() => toggle(tweak.id)}
                        aria-label={tweak.title[locale]}
                        className="mt-1 h-4 w-4 shrink-0 cursor-pointer accent-[var(--color-tsu-600)]"
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-sm font-medium">
                            {tweak.title[locale]}
                          </span>
                          <SafetyBadge safety={tweak.safety} locale={locale} compact />
                          {tweak.recommended && <Tag>★</Tag>}
                          {tweak.requiresRestart && (
                            <Tag title={t("tweaks.requiresRestart")}>
                              ⟳ {t("tweaks.requiresRestart")}
                            </Tag>
                          )}
                          {tweak.revert.length === 0 && (
                            <Tag title={t("tweaks.oneWay")}>⚠ {t("tweaks.oneWay")}</Tag>
                          )}
                        </div>
                        <p
                          className="mt-0.5 text-[12px]"
                          style={{ color: "var(--text-secondary)" }}
                        >
                          {tweak.description[locale]}
                        </p>
                        {outcome && (
                          <p
                            className="selectable mt-1 font-mono text-[11px]"
                            style={{
                              color: outcome.ok ? "var(--color-tsu-500)" : "#b3261e",
                            }}
                          >
                            {outcome.detail}
                          </p>
                        )}
                      </div>
                      <code
                        className="selectable shrink-0 text-[11px]"
                        style={{ color: "var(--text-muted)" }}
                      >
                        {tweak.id}
                      </code>
                    </li>
                  );
                })}
              </ul>
            </Panel>
          </section>
        ))}

        {grouped.length === 0 && (
          <Notice tone="info" title={t("activity.empty")} />
        )}
      </div>
    </div>
  );
}
