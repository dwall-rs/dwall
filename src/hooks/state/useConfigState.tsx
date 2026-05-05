import { createResource } from "solid-js";

import { readConfigFile } from "~/commands";

export const useConfigState = () => {
  const [data, { refetch, mutate }] = createResource<Config>(readConfigFile);

  return {
    data,
    refetch,
    mutate,
  };
};
