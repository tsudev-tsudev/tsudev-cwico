/**
 * The tsudev logo and wordmark.
 *
 * One component, used everywhere a brand mark belongs — the header, the about
 * panel, the empty state and the footer — so the mark is identical in all of
 * them and there is exactly one place that knows how to open tsudev.com.
 *
 * The wordmark is two coloured spans rather than an image: `tsu` in the brand
 * blue and `dev` in the brand orange, both taken from the logo itself. Text
 * scales with the user's font size, stays crisp at any DPI, and remains
 * selectable and searchable — none of which a baked-in PNG would give.
 */

import { openProductSite } from "../api";

export type BrandSize = "sm" | "md" | "lg";

const SIZES: Record<BrandSize, { logo: string; word: string; gap: string }> = {
  sm: { logo: "h-6 w-6", word: "text-[15px]", gap: "gap-1.5" },
  md: { logo: "h-9 w-9", word: "text-xl", gap: "gap-2.5" },
  lg: { logo: "h-16 w-16", word: "text-4xl", gap: "gap-4" },
};

interface BrandProps {
  size?: BrandSize;
  /** Show the product name under the wordmark. */
  tagline?: string;
  className?: string;
}

export function Brand({ size = "md", tagline, className = "" }: BrandProps) {
  const scale = SIZES[size];

  const open = (event: React.MouseEvent | React.KeyboardEvent) => {
    event.preventDefault();
    void openProductSite();
  };

  return (
    <a
      href="https://tsudev.com"
      onClick={open}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") open(event);
      }}
      title="tsudev.com"
      aria-label="tsudev — mở tsudev.com / open tsudev.com"
      className={`group inline-flex items-center ${scale.gap} rounded-lg px-1 py-0.5 no-underline transition-opacity hover:opacity-85 ${className}`}
    >
      <img
        src="./brand/tsudev-logo.png"
        alt=""
        aria-hidden="true"
        width={512}
        height={512}
        className={`${scale.logo} shrink-0 object-contain drop-shadow-sm transition-transform duration-200 group-hover:scale-105`}
      />
      <span className={`${scale.word} font-semibold leading-none tracking-tight`}>
        <span style={{ color: "var(--tsu-word)" }}>tsu</span>
        <span style={{ color: "var(--dev-word)" }}>dev</span>
      </span>
      {tagline && (
        <span
          className="ml-1 hidden text-[13px] font-normal sm:inline"
          style={{ color: "var(--text-muted)" }}
        >
          {tagline}
        </span>
      )}
    </a>
  );
}
