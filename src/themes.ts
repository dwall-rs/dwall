const thumbnails_base_url =
  "https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/";

const thumbnails_count = {
  "Big Sur": 8,
  "Big Sur 1": 16,
  Catalina: 8,
  "Earth ISS": 16,
  "Earth View": 16,
  Mojave: 16,
  "Monterey Bay 1": 16,
  "Monterey Graphic": 8,
  "Solar Gradients": 16,
  "The Beach": 8,
  "Ventura Graphic": 5,
};

export const themes: ThemeItem[] = Object.entries(thumbnails_count)
  .map(([id, count]) => ({
    id,
    thumbnail: Array.from(
      { length: count },
      (_, i) => `${thumbnails_base_url}${id.replace(" ", "")}/${i + 1}.avif`,
    ),
  }))
  .sort((a, b) => (a.id > b.id ? 1 : -1));
