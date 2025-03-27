export const generateGitHubThumbnailMirrorUrl = (
  originalUrl: string,
  mirrorTemplate?: string,
): string => {
  if (!mirrorTemplate) return originalUrl;

  const baseMirrorPath = mirrorTemplate.slice(
    0,
    mirrorTemplate.indexOf("<repo>") + "<repo>".length,
  );

  const originalPath = originalUrl.slice(originalUrl.indexOf("/blob/"));

  return (
    baseMirrorPath
      .replace("<owner>", "dwall-rs")
      .replace("<repo>", "dwall-assets") + originalPath
  );
};
