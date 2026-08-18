/**
 * Small shared building blocks.
 *
 * Every colour here comes from a CSS custom property defined in `index.css`,
 * never a hard-coded hex, so light and dark stay in step without a second set
 * of class names.
 */

import type { ReactNode } from "react";
import type { SafetyClass } from "../types";
import { safetyBlurb, safetyLabel } from "../i18n";
import type { Locale } from "../types";

/* -------------------------------------------------------------- Panel --- */

export function Panel({
  children,
  className = "",
  padded = true,
}: {
  children: ReactNode;
  className?: string;
  padded?: boolean;
}) {
  return (
    <section
      className={`rounded-xl border ${padded ? "p-4" : ""} ${className}`}
      style={{
        background: "var(--surface-panel)",
        borderColor: "var(--border-subtle)",
        boxShadow: "var(--shadow-panel)",
      }}
    >
      {children}
    </section>
  );
}

/* ------------------------------------------------------------- Button --- */

type ButtonVariant = "primary" | "danger" | "ghost" | "outline";

const BUTTON_BASE =
  "inline-flex items-center justify-center gap-2 rounded-lg px-3.5 py-2 text-sm font-medium " +
  "transition-colors disabled:cursor-not-allowed disabled:opacity-45";

export function Button({
  children,
  onClick,
  variant = "outline",
  disabled,
  title,
  className = "",
  type = "button",
}: {
  children: ReactNode;
  onClick?: () => void;
  variant?: ButtonVariant;
  disabled?: boolean;
  title?: string;
  className?: string;
  type?: "button" | "submit";
}) {
  const styles: Record<ButtonVariant, React.CSSProperties> = {
    primary: {
      background: "var(--color-tsu-600)",
      color: "#fff",
      borderColor: "var(--color-tsu-600)",
    },
    danger: {
      background: "var(--color-dev-600)",
      color: "#fff",
      borderColor: "var(--color-dev-600)",
    },
    outline: {
      background: "transparent",
      color: "var(--text-primary)",
      borderColor: "var(--border-strong)",
    },
    ghost: {
      background: "transparent",
      color: "var(--text-secondary)",
      borderColor: "transparent",
    },
  };

  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`${BUTTON_BASE} border ${className}`}
      style={styles[variant]}
    >
      {children}
    </button>
  );
}

/* -------------------------------------------------------- SafetyBadge --- */

/**
 * Colour is never the only signal: each class also carries a distinct glyph
 * and its own word, so the badge survives a monochrome screenshot and does
 * not depend on the viewer distinguishing green from amber.
 */
const SAFETY_STYLE: Record<
  SafetyClass,
  { glyph: string; fg: string; bg: string; border: string }
> = {
  safe: {
    glyph: "✓",
    fg: "var(--color-tsu-700)",
    bg: "color-mix(in srgb, var(--color-tsu-500) 12%, transparent)",
    border: "color-mix(in srgb, var(--color-tsu-500) 35%, transparent)",
  },
  caution: {
    glyph: "!",
    fg: "var(--color-dev-700)",
    bg: "color-mix(in srgb, var(--color-dev-500) 14%, transparent)",
    border: "color-mix(in srgb, var(--color-dev-500) 40%, transparent)",
  },
  unknown: {
    glyph: "?",
    fg: "var(--text-secondary)",
    bg: "color-mix(in srgb, var(--text-secondary) 10%, transparent)",
    border: "var(--border-strong)",
  },
  critical: {
    glyph: "⛊",
    fg: "#b3261e",
    bg: "color-mix(in srgb, #b3261e 12%, transparent)",
    border: "color-mix(in srgb, #b3261e 38%, transparent)",
  },
};

export function SafetyBadge({
  safety,
  locale,
  compact = false,
}: {
  safety: SafetyClass;
  locale: Locale;
  compact?: boolean;
}) {
  const style = SAFETY_STYLE[safety];
  return (
    <span
      title={safetyBlurb(locale, safety)}
      className={`inline-flex shrink-0 items-center gap-1 rounded-md border font-medium ${
        compact ? "px-1.5 py-0.5 text-[11px]" : "px-2 py-0.5 text-xs"
      }`}
      style={{ color: style.fg, background: style.bg, borderColor: style.border }}
    >
      <span aria-hidden="true">{style.glyph}</span>
      {safetyLabel(locale, safety)}
    </span>
  );
}

/* ---------------------------------------------------------------- Tag --- */

export function Tag({ children, title }: { children: ReactNode; title?: string }) {
  return (
    <span
      title={title}
      className="inline-flex items-center rounded px-1.5 py-0.5 text-[11px]"
      style={{
        background: "var(--surface-sunken)",
        color: "var(--text-secondary)",
      }}
    >
      {children}
    </span>
  );
}

/* -------------------------------------------------------------- Modal --- */

export function Modal({
  open,
  title,
  onClose,
  children,
  footer,
  width = "max-w-3xl",
}: {
  open: boolean;
  title: ReactNode;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  width?: string;
}) {
  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-6"
      style={{ background: "rgb(4 12 20 / 0.55)" }}
      onClick={onClose}
      role="presentation"
    >
      <div
        role="dialog"
        aria-modal="true"
        className={`flex max-h-[85vh] w-full ${width} flex-col overflow-hidden rounded-xl border`}
        style={{
          background: "var(--surface-panel)",
          borderColor: "var(--border-strong)",
          boxShadow: "0 24px 64px rgb(0 0 0 / 0.35)",
        }}
        onClick={(event) => event.stopPropagation()}
      >
        <header
          className="flex items-center justify-between border-b px-5 py-3.5"
          style={{ borderColor: "var(--border-subtle)" }}
        >
          <h2 className="text-base font-semibold">{title}</h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="rounded px-2 py-1 text-lg leading-none"
            style={{ color: "var(--text-muted)" }}
          >
            ×
          </button>
        </header>
        <div className="flex-1 overflow-y-auto px-5 py-4">{children}</div>
        {footer && (
          <footer
            className="flex items-center justify-end gap-2 border-t px-5 py-3"
            style={{
              borderColor: "var(--border-subtle)",
              background: "var(--surface-sunken)",
            }}
          >
            {footer}
          </footer>
        )}
      </div>
    </div>
  );
}

/* ------------------------------------------------------------- Notice --- */

export function Notice({
  tone,
  title,
  children,
  action,
}: {
  tone: "info" | "warning" | "danger" | "success";
  title: ReactNode;
  children?: ReactNode;
  action?: ReactNode;
}) {
  const tones = {
    info: { accent: "var(--color-tsu-500)", glyph: "i" },
    success: { accent: "var(--color-tsu-500)", glyph: "✓" },
    warning: { accent: "var(--color-dev-500)", glyph: "!" },
    danger: { accent: "#b3261e", glyph: "⛊" },
  } as const;
  const { accent, glyph } = tones[tone];

  return (
    <div
      className="flex items-start gap-3 rounded-lg border px-3.5 py-3"
      style={{
        borderColor: `color-mix(in srgb, ${accent} 35%, transparent)`,
        background: `color-mix(in srgb, ${accent} 8%, transparent)`,
      }}
    >
      <span
        aria-hidden="true"
        className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-[11px] font-bold"
        style={{ background: accent, color: "#fff" }}
      >
        {glyph}
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold">{title}</p>
        {children && (
          <div className="mt-1 text-[13px]" style={{ color: "var(--text-secondary)" }}>
            {children}
          </div>
        )}
      </div>
      {action && <div className="shrink-0">{action}</div>}
    </div>
  );
}

/* ------------------------------------------------------------ Spinner --- */

export function ProgressBar({ value }: { value?: number }) {
  const indeterminate = value === undefined;
  return (
    <div
      className="h-1.5 w-full overflow-hidden rounded-full"
      style={{ background: "var(--surface-sunken)" }}
      role="progressbar"
      aria-valuenow={indeterminate ? undefined : Math.round(value * 100)}
    >
      <div
        className={`h-full rounded-full ${indeterminate ? "w-1/4 animate-sweep" : ""}`}
        style={{
          background: "var(--color-tsu-500)",
          width: indeterminate ? undefined : `${Math.round(value * 100)}%`,
          transition: indeterminate ? undefined : "width 200ms ease-out",
        }}
      />
    </div>
  );
}
