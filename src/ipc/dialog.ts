import { open } from "@tauri-apps/plugin-dialog";

/** Open a native folder picker; returns the chosen path, or null if cancelled. */
export const pickDirectory = async (): Promise<string | null> => {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
};

/** Open a native image picker; returns the chosen path, or null if cancelled. */
export const pickImage = async (): Promise<string | null> => {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["jpg", "jpeg", "png", "webp", "bmp"],
      },
    ],
  });
  return typeof selected === "string" ? selected : null;
};
