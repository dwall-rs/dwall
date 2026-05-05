export interface MarkdownProps {
  content: string;
}

interface H2Tag {
  type: "h2";
  content: string;
}

interface ListTag {
  type: "ul";
  items: string[]; // li
}

export interface BlockCodeTag {
  type: "pre";
  content: string;
  language: "typescript" | "javascript" | "rust" | "css" | "text";
}

interface TextTag {
  type: "text";
  content: string;
}

export type MarkdownTag = H2Tag | ListTag | BlockCodeTag | TextTag;
