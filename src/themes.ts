const images = {
  Catalina: Object.values(
    import.meta.glob("~/assets/thumbnail/Catalina/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
  "Big Sur": Object.values(
    import.meta.glob("~/assets/thumbnail/BigSur/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
  Mojave: Object.values(
    import.meta.glob("~/assets/thumbnail/Mojave/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
  "Big Sur 1": Object.values(
    import.meta.glob("~/assets/thumbnail/BigSur1/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
  "Earth ISS": Object.values(
    import.meta.glob("~/assets/thumbnail/EarthISS/*.avif", {
      import: "default",
      eager: true,
    }) as Record<string, string>,
  ),
};

export const themes: ThemeItem[] = Object.entries(images)
  .map(([id, thumbnails]) => ({
    id,
    thumbnail: thumbnails,
  }))
  .sort((a, b) => (a.id > b.id ? 1 : -1));
