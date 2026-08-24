/**
 * The per-item acknowledgement for Caution and Unknown items.
 *
 * The engine refuses to plan these without `confirmed: true`, and this dialog
 * is the only thing in the UI that sets it. Ticking a checkbox is not the
 * acknowledgement - reading what the item costs and saying yes is. That is
 * the difference between a user who chose to lose their default photo viewer
 * and one who is surprised by it a week later.
 */

import type { Locale, SoftwareItem } from "../types";
import { kindLabel, translator } from "../i18n";
import { Button, Modal, Notice, SafetyBadge } from "./primitives";

export function ConfirmCaution({
  item,
  locale,
  onConfirm,
  onCancel,
}: {
  item: SoftwareItem | null;
  locale: Locale;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const t = translator(locale);
  if (!item) return null;

  return (
    <Modal
      open
      title={t("confirm.caution.title")}
      onClose={onCancel}
      width="max-w-xl"
      footer={
        <>
          <Button variant="ghost" onClick={onCancel}>
            {t("action.cancel")}
          </Button>
          <Button variant="danger" onClick={onConfirm}>
            {t("action.confirm")}
          </Button>
        </>
      }
    >
      <div className="mb-4 flex flex-wrap items-center gap-2">
        <SafetyBadge safety={item.safety} locale={locale} />
        <span className="text-sm font-semibold">{item.name}</span>
        <span className="text-[12px]" style={{ color: "var(--text-muted)" }}>
          {kindLabel(locale, item.source)}
          {item.publisher ? ` · ${item.publisher}` : ""}
        </span>
      </div>

      <Notice
        tone={item.safety === "unknown" ? "info" : "warning"}
        title={t("confirm.caution.body")}
      >
        {item.safetyReason?.[locale]}
      </Notice>

      {item.description && (
        <p className="mt-3 text-[13px]" style={{ color: "var(--text-secondary)" }}>
          {item.description[locale]}
        </p>
      )}

      {(item.installLocation || item.registryKey) && (
        <dl className="mt-4 space-y-2">
          {item.installLocation && (
            <div>
              <dt
                className="text-[11px] font-semibold uppercase tracking-wide"
                style={{ color: "var(--text-muted)" }}
              >
                {t("detail.location")}
              </dt>
              <dd className="selectable break-all font-mono text-[12px]">
                {item.installLocation}
              </dd>
            </div>
          )}
          {item.registryKey && (
            <div>
              <dt
                className="text-[11px] font-semibold uppercase tracking-wide"
                style={{ color: "var(--text-muted)" }}
              >
                {t("detail.registryKey")}
              </dt>
              <dd className="selectable break-all font-mono text-[12px]">
                {item.registryKey}
              </dd>
            </div>
          )}
        </dl>
      )}
    </Modal>
  );
}
