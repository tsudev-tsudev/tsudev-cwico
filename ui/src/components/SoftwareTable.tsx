/**
 * The inventory: toolbar, table, and the detail drawer.
 *
 * The table is the screen users spend their time in, so two decisions matter:
 *
 * * A `critical` row has no checkbox at all. Rendering a disabled checkbox
 *   would invite clicking it and wondering why nothing happens; rendering a
 *   lock says "this is not on offer" the first time.
 * * The safety reason is on the row, not hidden behind a tooltip. A user
 *   deciding whether to remove Microsoft Edge should not have to hover to
 *   learn that Windows renders PDFs with it.
 */

import { useMemo, useState } from "react";
import type { Locale, SafetyClass, SoftwareItem, SourceKind } from "../types";
import { formatBytes, kindLabel, safetyLabel, translator } from "../i18n";
import { Button, Panel, SafetyBadge, Tag } from "./primitives";

export type SortBy = "name" | "size" | "safety" | "publisher" | "kind" | "installDate";

const SAFETY_ORDER: Record<SafetyClass, number> = {
  safe: 0,
  caution: 1,
  unknown: 2,
  critical: 3,
};

const KINDS: SourceKind[] = [
  "registry_uninstall",
  "appx_package",
  "appx_provisioned",
  "windows_service",
  "scheduled_task",
  "startup_entry",
];

const SAFETIES: SafetyClass[] = ["safe", "caution", "unknown", "critical"];

interface Props {
  items: SoftwareItem[];
  locale: Locale;
  selected: Set<string>;
  confirmed: Set<string>;
  onToggle: (item: SoftwareItem) => void;
  onSelectAllSafe: () => void;
  onClearSelection: () => void;
}

export function SoftwareTable({
  items,
  locale,
  selected,
  confirmed,
  onToggle,
  onSelectAllSafe,
  onClearSelection,
}: Props) {
  const t = translator(locale);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<SourceKind | "">("");
  const [safety, setSafety] = useState<SafetyClass | "">("");
  const [sortBy, setSortBy] = useState<SortBy>("safety");
  const [detail, setDetail] = useState<SoftwareItem | null>(null);

  const visible = useMemo(() => {
    const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    const filtered = items.filter((item) => {
      if (kind && item.source !== kind) return false;
      if (safety && item.safety !== safety) return false;
      if (terms.length) {
        const haystack = [
          item.name,
          item.publisher,
          item.version,
          item.systemName,
          item.packageFamilyName,
        ]
          .filter(Boolean)
          .join(" ")
          .toLowerCase();
        if (!terms.every((term) => haystack.includes(term))) return false;
      }
      return true;
    });

    const collator = new Intl.Collator(locale === "vi" ? "vi" : "en", {
      sensitivity: "base",
    });
    return filtered.sort((a, b) => {
      switch (sortBy) {
        case "size":
          return (b.sizeBytes ?? 0) - (a.sizeBytes ?? 0);
        case "safety":
          return (
            SAFETY_ORDER[a.safety] - SAFETY_ORDER[b.safety] ||
            collator.compare(a.name, b.name)
          );
        case "publisher":
          return (
            collator.compare(a.publisher ?? "", b.publisher ?? "") ||
            collator.compare(a.name, b.name)
          );
        case "kind":
          return a.source.localeCompare(b.source) || collator.compare(a.name, b.name);
        case "installDate":
          return (b.installDate ?? "").localeCompare(a.installDate ?? "");
        default:
          return collator.compare(a.name, b.name);
      }
    });
  }, [items, query, kind, safety, sortBy, locale]);

  const selectableCount = items.filter((i) => i.safety === "safe").length;

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      {/* ---------------------------------------------------------- toolbar */}
      <div className="flex flex-wrap items-center gap-2">
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t("filter.search")}
          className="selectable min-w-[220px] flex-1 rounded-lg border px-3 py-2 text-sm"
          style={{
            background: "var(--surface-sunken)",
            borderColor: "var(--border-subtle)",
            color: "var(--text-primary)",
          }}
        />
        <Select
          value={kind}
          onChange={(value) => setKind(value as SourceKind | "")}
          options={[
            { value: "", label: t("filter.allKinds") },
            ...KINDS.map((k) => ({ value: k, label: kindLabel(locale, k) })),
          ]}
        />
        <Select
          value={safety}
          onChange={(value) => setSafety(value as SafetyClass | "")}
          options={[
            { value: "", label: t("filter.allSafety") },
            ...SAFETIES.map((s) => ({ value: s, label: safetyLabel(locale, s) })),
          ]}
        />
        <Select
          value={sortBy}
          onChange={(value) => setSortBy(value as SortBy)}
          options={[
            { value: "safety", label: t("sort.safety") },
            { value: "name", label: t("sort.name") },
            { value: "size", label: t("sort.size") },
            { value: "publisher", label: t("sort.publisher") },
            { value: "kind", label: t("sort.kind") },
            { value: "installDate", label: t("sort.installDate") },
          ]}
        />
      </div>

      <div className="flex flex-wrap items-center gap-2 text-[13px]">
        <Button variant="ghost" onClick={onSelectAllSafe} disabled={!selectableCount}>
          {t("select.all")} ({selectableCount})
        </Button>
        <Button variant="ghost" onClick={onClearSelection} disabled={!selected.size}>
          {t("select.none")}
        </Button>
        <span style={{ color: "var(--text-muted)" }}>
          {visible.length} / {items.length} {t("scan.itemsFound")}
        </span>
      </div>

      {/* ------------------------------------------------------------ table */}
      <Panel padded={false} className="min-h-0 flex-1 overflow-y-auto">
        {visible.length === 0 ? (
          <p className="p-8 text-center text-sm" style={{ color: "var(--text-muted)" }}>
            {t("scan.noResults")}
          </p>
        ) : (
          <ul>
            {visible.map((item) => {
              const isSelected = selected.has(item.id);
              const isBlocked = item.safety === "critical";
              const needsConfirm =
                item.safety === "caution" || item.safety === "unknown";
              return (
                <li
                  key={item.id}
                  className="flex items-start gap-3 border-b px-3.5 py-2.5 last:border-b-0"
                  style={{
                    borderColor: "var(--border-subtle)",
                    background: isSelected
                      ? "color-mix(in srgb, var(--color-tsu-500) 8%, transparent)"
                      : undefined,
                  }}
                >
                  <div className="flex h-5 w-5 shrink-0 items-center justify-center pt-0.5">
                    {isBlocked ? (
                      <span
                        aria-hidden="true"
                        title={t("select.blocked")}
                        style={{ color: "var(--text-muted)" }}
                      >
                        🔒
                      </span>
                    ) : (
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => onToggle(item)}
                        aria-label={item.name}
                        className="h-4 w-4 cursor-pointer accent-[var(--color-tsu-600)]"
                      />
                    )}
                  </div>

                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="truncate text-sm font-medium">{item.name}</span>
                      <SafetyBadge safety={item.safety} locale={locale} compact />
                      <Tag>{kindLabel(locale, item.source)}</Tag>
                      {item.extra.running === "1" && (
                        <Tag title={t("detail.running")}>● {t("detail.running")}</Tag>
                      )}
                      {item.extra.provisioned === "1" && (
                        <Tag>{kindLabel(locale, "appx_provisioned")}</Tag>
                      )}
                      {isSelected && needsConfirm && !confirmed.has(item.id) && (
                        <Tag title={t("confirm.caution.body")}>
                          ⚠ {t("confirm.caution.title")}
                        </Tag>
                      )}
                    </div>

                    <p
                      className="mt-0.5 truncate text-[12px]"
                      style={{ color: "var(--text-muted)" }}
                    >
                      {[item.publisher, item.version].filter(Boolean).join(" · ") || "—"}
                    </p>

                    {item.safetyReason && (
                      <p
                        className="mt-1 line-clamp-2 text-[12px]"
                        style={{ color: "var(--text-secondary)" }}
                      >
                        {item.safetyReason[locale]}
                      </p>
                    )}
                  </div>

                  <div className="flex shrink-0 items-center gap-3 pt-0.5">
                    <span
                      className="w-20 text-right text-[13px] tabular-nums"
                      style={{ color: "var(--text-secondary)" }}
                    >
                      {formatBytes(locale, item.sizeBytes)}
                    </span>
                    <button
                      type="button"
                      onClick={() => setDetail(item)}
                      className="rounded px-2 py-1 text-[12px]"
                      style={{ color: "var(--color-tsu-500)" }}
                    >
                      {t("action.details")}
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </Panel>

      {detail && (
        <DetailDrawer item={detail} locale={locale} onClose={() => setDetail(null)} />
      )}
    </div>
  );
}

function Select({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className="rounded-lg border px-2.5 py-2 text-sm"
      style={{
        background: "var(--surface-sunken)",
        borderColor: "var(--border-subtle)",
        color: "var(--text-primary)",
      }}
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}

function DetailDrawer({
  item,
  locale,
  onClose,
}: {
  item: SoftwareItem;
  locale: Locale;
  onClose: () => void;
}) {
  const t = translator(locale);
  const rows: [string, string | undefined][] = [
    [t("detail.publisher"), item.publisher],
    [t("detail.version"), item.version],
    [t("detail.size"), item.sizeBytes ? formatBytes(locale, item.sizeBytes) : undefined],
    [t("detail.installed"), item.installDate],
    [t("detail.location"), item.installLocation],
    [t("detail.registryKey"), item.registryKey],
    [t("detail.package"), item.packageFullName ?? item.packageFamilyName],
    [t("detail.service"), item.systemName],
    [t("detail.uninstallString"), item.uninstallString],
    [t("detail.quietUninstall"), item.quietUninstallString],
    [t("detail.processes"), item.executables.join(", ") || undefined],
  ];

  return (
    <div
      className="fixed inset-y-0 right-0 z-40 flex w-full max-w-md flex-col border-l"
      style={{
        background: "var(--surface-panel)",
        borderColor: "var(--border-strong)",
        boxShadow: "-16px 0 48px rgb(0 0 0 / 0.18)",
      }}
    >
      <header
        className="flex items-start justify-between gap-3 border-b px-5 py-4"
        style={{ borderColor: "var(--border-subtle)" }}
      >
        <div className="min-w-0">
          <h3 className="text-sm font-semibold">{item.name}</h3>
          <div className="mt-1.5">
            <SafetyBadge safety={item.safety} locale={locale} />
          </div>
        </div>
        <button
          type="button"
          onClick={onClose}
          aria-label={t("action.close")}
          className="rounded px-2 text-lg leading-none"
          style={{ color: "var(--text-muted)" }}
        >
          ×
        </button>
      </header>

      <div className="flex-1 overflow-y-auto px-5 py-4">
        {item.safetyReason && (
          <div className="mb-4">
            <p
              className="mb-1 text-[11px] font-semibold uppercase tracking-wide"
              style={{ color: "var(--text-muted)" }}
            >
              {t("detail.why")}
            </p>
            <p className="text-[13px]" style={{ color: "var(--text-secondary)" }}>
              {item.safetyReason[locale]}
            </p>
          </div>
        )}

        {item.description && (
          <p className="mb-4 text-[13px]" style={{ color: "var(--text-secondary)" }}>
            {item.description[locale]}
          </p>
        )}

        <dl className="space-y-2.5">
          {rows
            .filter(([, value]) => Boolean(value))
            .map(([label, value]) => (
              <div key={label}>
                <dt
                  className="text-[11px] font-semibold uppercase tracking-wide"
                  style={{ color: "var(--text-muted)" }}
                >
                  {label}
                </dt>
                <dd className="selectable mt-0.5 break-all font-mono text-[12px]">
                  {value}
                </dd>
              </div>
            ))}
        </dl>
      </div>
    </div>
  );
}
