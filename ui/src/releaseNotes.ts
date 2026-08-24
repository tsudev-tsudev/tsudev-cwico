/**
 * Render release notes for the update dialog.
 *
 * The notes come from `latest.json`, which carries the GitHub release body -
 * which is the project's CHANGELOG section, which is Markdown. The dialog is a
 * small panel inside a modal, not a document viewer, so pulling in a Markdown
 * renderer for it would be disproportionate: the whole front end is 250 kB and
 * a renderer is a meaningful fraction of that, for one box.
 *
 * Instead the common Markdown noise is reduced to readable prose. The goal is
 * that a user blocked from their tool can read *why* without meeting a stray
 * `>` or a row of backticks - not to reproduce the CHANGELOG faithfully. The
 * full text stays one click away on the release page.
 */

export interface NotesBlock {
  kind: "text" | "code" | "bullet";
  text: string;
}

const HEADING = /^#{1,6}\s+/;
const BLOCKQUOTE = /^>\s?/;
const BULLET = /^[-*+]\s+/;
const NUMBERED = /^\d+\.\s+/;
const FENCE = /^\s*```/;
const HORIZONTAL_RULE = /^\s*([-*_])\1{2,}\s*$/;

/** Inline markers, removed in an order that does not eat the wrong thing. */
function stripInline(line: string): string {
  return (
    line
      // Links: `[label](url)` keeps the label, which is what carries meaning.
      .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
      // Images add nothing here.
      .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
      // Bold and italics, longest marker first so `**x**` does not become `*x*`.
      .replace(/\*\*\*([^*]+)\*\*\*/g, "$1")
      .replace(/\*\*([^*]+)\*\*/g, "$1")
      .replace(/(^|[^*])\*([^*]+)\*/g, "$1$2")
      .replace(/__([^_]+)__/g, "$1")
      // Inline code: the backticks go, the identifier stays.
      .replace(/`([^`]+)`/g, "$1")
      .trimEnd()
  );
}

/**
 * Split notes into blocks the dialog can style: prose, bullets, and code,
 * which stays monospaced because a command the user is told to run should
 * look like one.
 */
export function parseNotes(notes: string): NotesBlock[] {
  const blocks: NotesBlock[] = [];
  let paragraph: string[] = [];
  let code: string[] = [];
  let inCode = false;

  const flushParagraph = () => {
    if (paragraph.length) {
      blocks.push({ kind: "text", text: paragraph.join(" ").trim() });
      paragraph = [];
    }
  };

  // Tracks whether the previous block was a bullet, so a wrapped line
  // continues it rather than becoming a stray paragraph.
  let lastWasBullet = false;

  for (const raw of notes.replace(/\r\n/g, "\n").split("\n")) {
    const line = raw.replace(BLOCKQUOTE, "");
    const isIndented = /^\s{2,}\S/.test(line);

    if (FENCE.test(line)) {
      lastWasBullet = false;
      if (inCode) {
        blocks.push({ kind: "code", text: code.join("\n").trim() });
        code = [];
      } else {
        flushParagraph();
      }
      inCode = !inCode;
      continue;
    }
    if (inCode) {
      code.push(line);
      continue;
    }

    const trimmed = line.trim();
    if (!trimmed || HORIZONTAL_RULE.test(trimmed)) {
      flushParagraph();
      lastWasBullet = false;
      continue;
    }
    if (BULLET.test(trimmed) || NUMBERED.test(trimmed)) {
      flushParagraph();
      blocks.push({
        kind: "bullet",
        text: stripInline(trimmed.replace(BULLET, "").replace(NUMBERED, "")),
      });
      lastWasBullet = true;
      continue;
    }
    if (HEADING.test(trimmed)) {
      flushParagraph();
      blocks.push({ kind: "text", text: stripInline(trimmed.replace(HEADING, "")) });
      lastWasBullet = false;
      continue;
    }
    // A wrapped bullet: Markdown continues a list item on an indented line,
    // and splitting it into its own paragraph reads as a non-sequitur.
    if (lastWasBullet && isIndented && !paragraph.length) {
      const previous = blocks[blocks.length - 1];
      if (previous && previous.kind === "bullet") {
        previous.text = `${previous.text} ${stripInline(trimmed)}`.trim();
        continue;
      }
    }
    lastWasBullet = false;
    paragraph.push(stripInline(trimmed));
  }

  flushParagraph();
  if (code.length) {
    // An unterminated fence: keep the content rather than dropping it.
    blocks.push({ kind: "code", text: code.join("\n").trim() });
  }
  return blocks.filter((block) => block.text.length > 0);
}
