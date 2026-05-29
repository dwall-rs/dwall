import type { JSXElement } from "solid-js";

export interface ImageData {
  width: number;
  height: number;
  top: number;
  bottom: number;
  left: number;
  right: number;
}

export interface ImageProps {
  themeID: string;
  serialNumber: number;
  src: string;
  isLocal?: boolean;
  alt?: string;
  width?: number;
  height?: number;
  class?: string;
  onLoad?: (data: ImageData) => void;
  onError?: (error: Error) => void;
  skeleton?: JSXElement;
  fallbackSrc?: string;
  retryCount?: number;
}
