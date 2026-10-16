// Responsibility: the random candidate pool's pure rule — the pool saved in the config versus what can actually be applied. No store access.

/**
 * The pool to start from, given the pool saved in the config and the ids that can be
 * applied now.
 *
 * The configured pool is the source of truth, minus ids the catalog no longer lists and
 * themes whose frames are not on disk: neither could ever be applied, so neither belongs
 * in the pool. `null` means the config has no pool yet, which means "everything that is
 * available".
 */
export const seedPool = (
  configured: string[] | null,
  availableIds: string[],
): string[] => {
  if (!configured) return [...availableIds];

  const available = new Set(availableIds);
  return configured.filter((id) => available.has(id));
};
