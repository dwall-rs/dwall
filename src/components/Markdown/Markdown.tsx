// Markdown.tsx
import { children, type Component, createMemo, For } from "solid-js";
import type { MarkdownProps, MarkdownTag } from "./Markdown.types";

import { useMarkdownParser } from "./useMarkdownParser";

import * as styles from "./Markdown.css";

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
    parsedContent().map((item) => {
      switch (item.type) {
        case "h2":
          return <h2 class={styles.h2}>{item.content}</h2>;

        case "ul":
          return (
            <ul class={styles.ul}>
              <For each={item.items}>
                {(listItem) => <li class={styles.li} innerHTML={listItem} />}
              </For>
            </ul>
          );

        case "pre":
          return (
            <pre class={styles.blockCode}>
              <code
                class={`${styles.languageBase} ${styles.languages[item.language]}`}
              >
                {item.content}
              </code>
            </pre>
          );

        default: // plain text
          return <p class={styles.textBlock} innerHTML={item.content} />;
      }
    }),
  );

  return <div class={styles.markdown}>{content()}</div>;
};
