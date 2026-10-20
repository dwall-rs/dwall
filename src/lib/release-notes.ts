// Responsibility: the small slice of Markdown release notes actually use (headings, bullet lists,
// paragraphs, inline `code`), parsed into blocks so the view can render them as elements.

/** One line of inline text, flagged when it came from between backticks. */
export interface ReleaseNoteSpan {
  code: boolean;
  text: string;
}

export type ReleaseNoteBlock =
  | { kind: "heading"; text: string }
  | { kind: "list"; items: string[] }
  | { kind: "paragraph"; text: string };

const HEADING = /^#{1,6}\s+(.+)$/;
const BULLET = /^[-*]\s+(.+)$/;

/**
 * Splits release notes into blocks.
 *
 * Deliberately not a Markdown parser: the notes are a generated bullet list, and pulling in
 * a parser (or an HTML sink) for that would add a dependency and an injection surface. What
 * is not recognised stays visible as text, which is the safe default for a link or a stray
 * `*`.
 */
export function parseReleaseNotes(markdown: string): ReleaseNoteBlock[] {
  const blocks: ReleaseNoteBlock[] = [];
  let items: string[] = [];

  const flushList = () => {
    if (items.length === 0) return;

    blocks.push({ kind: "list", items });
    items = [];
  };

  for (const rawLine of markdown.split(/\r?\n/)) {
    const line = rawLine.trimEnd();

    if (line.trim() === "") {
      flushList();
      continue;
    }

    const heading = HEADING.exec(line);
    if (heading) {
      flushList();
      blocks.push({ kind: "heading", text: heading[1].trim() });
      continue;
    }

    const bullet = BULLET.exec(line);
    if (bullet) {
      items.push(bullet[1].trim());
      continue;
    }

    flushList();
    blocks.push({ kind: "paragraph", text: line.trim() });
  }

  flushList();

  return blocks;
}

/**
 * Splits a line into plain and `code` spans.
 *
 * The segments of `split("`")` alternate outside/inside, so an odd index is a code span. An
 * unbalanced backtick therefore swallows the rest of the line as code instead of being
 * dropped, which keeps the text visible.
 */
export function parseReleaseNoteSpans(text: string): ReleaseNoteSpan[] {
  return text
    .split("`")
    .map((part, index) => ({ code: index % 2 === 1, text: part }))
    .filter((span) => span.text !== "");
}
