// Markdown.tsx
import { children, type Component, createMemo, For, Show } from "solid-js";
import type { MarkdownProps, MarkdownTag } from "./Markdown.types";

import { useMarkdownParser } from "./useMarkdownParser";
import { Separator } from "../separator";

export const Markdown: Component<MarkdownProps> = (props) => {
  const parser = useMarkdownParser();

  // Parse Markdown content
  const parsedContent = createMemo(() => {
    const lines = props.content.split("\n");
    const result: Array<MarkdownTag> = [];
    let i = 0;

    while (i < lines.length) {
      const line = lines[i].trim();

      // 1. Parse headers
      const headerTag = parser.parseHeaders(line);
      if (headerTag) {
        result.push(headerTag);
        i++;
        continue;
      }

      // 2. Parse lists
      const { tag: listTag, nextIndex: listNextIndex } = parser.parseLists(
        lines,
        i,
      );
      if (listTag) {
        result.push(listTag);
        i = listNextIndex;
        continue;
      }

      // 3. Parse code blocks
      const { tag: codeTag, nextIndex: codeNextIndex } = parser.parseCodeBlocks(
        lines,
        i,
      );
      if (codeTag) {
        result.push(codeTag);
        i = codeNextIndex;
        continue;
      }

      // 4. Parse inline code
      const inlineCodeTag = parser.parseInlineCode(line);
      if (inlineCodeTag) {
        result.push(inlineCodeTag);
        i++;
        continue;
      }

      // 5. Parse plain text
      const textTag = parser.parsePlainText(line);
      if (textTag) {
        result.push(textTag);
      }

      i++;
    }

    return result;
  });

  const content = children(() =>
    parsedContent().map((item, index) => {
      switch (item.type) {
        case "h2":
          return (
            <>
              <Show when={index > 0 && parsedContent().length > 1}>
                <Separator class="my-3" />
              </Show>
              <h2 class="m-0 pb-1.5 font-bold text-base">{item.content}</h2>
            </>
          );

        case "ul":
          return (
            <ul class="my-1 mx-0 pl-2">
              <For each={item.items}>
                {(listItem) => (
                  <li
                    class="my-1 mx-0 ml-2 marker:content-['•']"
                    innerHTML={listItem}
                  />
                )}
              </For>
            </ul>
          );

        case "pre":
          return (
            <pre class="bg-neutral-100 rounded-md p-2 my-1 mx-0 overflow-auto font-mono">
              <code class="text-sm">{item.content}</code>
            </pre>
          );

        default: // plain text
          return <p class="my-1 mx-0" innerHTML={item.content} />;
      }
    }),
  );

  return <div class="mb-2 max-h-72 overflow-y-auto scrollbar">{content()}</div>;
};
