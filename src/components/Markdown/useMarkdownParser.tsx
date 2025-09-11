import * as styles from "./Markdown.css";
import type { BlockCodeTag, MarkdownTag } from "./Markdown.types";

// Utility function: HTML escaping
const escapeHtml = (text: string): string => {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
};

// Utility function: parse inline code
const parseInlineCode = (text: string): string => {
  const parts = text.split("`");
  let result = "";

  for (let i = 0; i < parts.length; i++) {
    if (i % 2 === 1) {
      // Odd positions are code content
      result += `<code class="${styles.inlineCode}">${escapeHtml(parts[i])}</code>`;
    } else {
      // Even positions are plain text
      result += escapeHtml(parts[i]);
    }
  }

  return result;
};

// Parser interface
interface MarkdownParser {
  parseHeaders: (line: string) => MarkdownTag | null;
  parseLists: (
    lines: string[],
    startIndex: number,
  ) => { tag: MarkdownTag | null; nextIndex: number };
  parseCodeBlocks: (
    lines: string[],
    startIndex: number,
  ) => { tag: MarkdownTag | null; nextIndex: number };
  parseInlineCode: (line: string) => MarkdownTag | null;
  parsePlainText: (line: string) => MarkdownTag | null;
}

// Create Markdown parser
export const useMarkdownParser = (): MarkdownParser => {
  return {
    parseHeaders: (line: string) => {
      if (line.startsWith("## ")) {
        return {
          type: "h2",
          content: line.substring(3).trim(),
        };
      }
      return null;
    },

    parseLists: (lines: string[], startIndex: number) => {
      const startLine = lines[startIndex].trim();
      if (!startLine.startsWith("* ") && !startLine.startsWith("- ")) {
        return { tag: null, nextIndex: startIndex };
      }

      const listItems: string[] = [];
      let i = startIndex;

      // Collect all consecutive list items
      while (i < lines.length) {
        const currentLine = lines[i].trim();
        if (!currentLine.startsWith("* ") && !currentLine.startsWith("- ")) {
          break;
        }

        let listItem = currentLine.substring(2).trim();
        // Handle inline code in list items
        if (listItem.includes("`")) {
          listItem = parseInlineCode(listItem);
        }

        listItems.push(listItem);
        i++;
      }

      return {
        tag: listItems.length > 0 ? { type: "ul", items: listItems } : null,
        nextIndex: i,
      };
    },

    parseCodeBlocks: (lines: string[], startIndex: number) => {
      const startLine = lines[startIndex].trim();
      if (!startLine.startsWith("```")) {
        return { tag: null, nextIndex: startIndex };
      }

      const language = startLine
        .substring(3)
        .trim() as BlockCodeTag["language"];
      const codeLines: string[] = [];
      let i = startIndex + 1;

      // Collect code content until end marker is found
      while (i < lines.length && !lines[i].trim().startsWith("```")) {
        codeLines.push(lines[i]);
        i++;
      }

      return {
        tag: {
          type: "pre",
          content: codeLines.join("\n"),
          language: language || "text",
        },
        nextIndex: i + 1, // Skip end marker
      };
    },

    parseInlineCode: (line: string) => {
      if (line.includes("`")) {
        const parsedLine = parseInlineCode(line);
        return {
          type: "text",
          content: parsedLine,
        };
      }
      return null;
    },

    parsePlainText: (line: string) => {
      if (line.trim()) {
        return {
          type: "text",
          content: escapeHtml(line.trim()),
        };
      }
      return null;
    },
  };
};
