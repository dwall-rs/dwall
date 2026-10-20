// Responsibility: renders the parsed release notes as elements — headings, lists, paragraphs and
// inline code — with no HTML sink anywhere.
import { For, Match, Switch, type JSX } from "solid-js";
import {
  parseReleaseNoteSpans,
  parseReleaseNotes,
  type ReleaseNoteBlock,
} from "@/lib/release-notes";

const asHeading = (block: ReleaseNoteBlock) =>
  block.kind === "heading" ? block : undefined;

const asList = (block: ReleaseNoteBlock) =>
  block.kind === "list" ? block : undefined;

const asParagraph = (block: ReleaseNoteBlock) =>
  block.kind === "paragraph" ? block : undefined;

function Spans(props: { text: string }): JSX.Element {
  return (
    <For each={parseReleaseNoteSpans(props.text)}>
      {(span) =>
        span.code ? (
          <code class="rounded bg-foreground/8 px-1 py-px font-mono text-[0.92em]">
            {span.text}
          </code>
        ) : (
          span.text
        )
      }
    </For>
  );
}

export function ReleaseNotes(props: { markdown: string }) {
  return (
    <div class="flex flex-col gap-2.5 text-[12.5px] leading-relaxed text-muted-foreground">
      <For each={parseReleaseNotes(props.markdown)}>
        {(block) => (
          <Switch>
            <Match when={asHeading(block)}>
              {(heading) => (
                <h3 class="font-display text-[13px] font-bold text-foreground">
                  {heading().text}
                </h3>
              )}
            </Match>
            <Match when={asList(block)}>
              {(list) => (
                <ul class="flex list-disc flex-col gap-1 pl-4 marker:text-primary/60">
                  <For each={list().items}>
                    {(item) => (
                      <li>
                        <Spans text={item} />
                      </li>
                    )}
                  </For>
                </ul>
              )}
            </Match>
            <Match when={asParagraph(block)}>
              {(paragraph) => (
                <p>
                  <Spans text={paragraph().text} />
                </p>
              )}
            </Match>
          </Switch>
        )}
      </For>
    </div>
  );
}
