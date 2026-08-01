import { useQuery } from "@tanstack/react-query";
import { providersApi } from "@/lib/api";

export const piKeys = {
  all: ["pi"] as const,
  liveProviderIds: ["pi", "liveProviderIds"] as const,
};

export function usePiLiveProviderIds(enabled: boolean) {
  return useQuery({
    queryKey: piKeys.liveProviderIds,
    queryFn: () => providersApi.getPiLiveProviderIds(),
    enabled,
  });
}
