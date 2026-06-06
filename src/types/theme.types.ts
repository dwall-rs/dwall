export type ImageFormat = "jpeg" | "png";

export interface CustomizedTheme {
  id: string;
  directory: string;
  image_format: ImageFormat;
  theme_name: string;
  author: string;
  thumbnails: string[];
  version: number;
}
