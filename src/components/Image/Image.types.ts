export interface ImageData {
  width: number;
  height: number;
}

export interface ImageProps {
  themeID: string;
  serialNumber: number;
  src: string;
  alt?: string;
  width?: number;
  height?: number;
  class?: string;
  onLoad?: (data: ImageData) => void;
  onError?: (error: Error) => void;
  fallbackSrc?: string;
  retryCount?: number;
}
