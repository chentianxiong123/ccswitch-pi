import { useState, useCallback, useMemo } from "react";
import type { AppId } from "@/lib/api";
import { useProvidersQuery } from "@/lib/query/queries";

export type PiApiMode = string;

export interface PiModel {
  id: string;
  name?: string;
  contextWindow?: number;
  maxTokens?: number;
  reasoning?: boolean;
  cost?: {
    input?: number;
    output?: number;
    cacheRead?: number;
    cacheWrite?: number;
  };
  [key: string]: unknown;
}

export interface PiProviderSettingsConfig {
  name?: string;
  baseUrl?: string;
  apiKey?: string;
  api?: PiApiMode;
  models?: PiModel[];
  headers?: Record<string, string>;
  [key: string]: unknown;
}

export const PI_AGENT_DEFAULT_API_MODE: PiApiMode = "openai-completions";

interface UsePiFormStateParams {
  initialData?: {
    settingsConfig?: Record<string, unknown>;
  };
  appId: AppId;
  providerId?: string;
  onSettingsConfigChange: (config: string) => void;
  getSettingsConfig: () => string;
}

const PI_AGENT_DEFAULT_CONFIG_OBJ = {
  name: "",
  baseUrl: "",
  apiKey: "",
} as const;

export const PI_AGENT_DEFAULT_CONFIG = JSON.stringify(
  PI_AGENT_DEFAULT_CONFIG_OBJ,
  null,
  2,
);

export interface PiFormState {
  piProviderKey: string;
  setPiProviderKey: (key: string) => void;
  piBaseUrl: string;
  piApiKey: string;
  piApi: PiApiMode;
  piModels: PiModel[];
  existingPiKeys: string[];
  handlePiBaseUrlChange: (baseUrl: string) => void;
  handlePiApiKeyChange: (apiKey: string) => void;
  handlePiApiChange: (api: PiApiMode) => void;
  handlePiModelsChange: (models: PiModel[]) => void;
  resetPiState: (config?: Partial<PiProviderSettingsConfig>) => void;
}

function parsePiField<T>(
  initialData: UsePiFormStateParams["initialData"],
  field: string,
  fallback: T,
): T {
  try {
    if (initialData?.settingsConfig) {
      return (initialData.settingsConfig[field] as T) || fallback;
    }
    return (
      ((PI_AGENT_DEFAULT_CONFIG_OBJ as Record<string, unknown>)[field] as T) ||
      fallback
    );
  } catch {
    return fallback;
  }
}

export function usePiFormState({
  initialData,
  appId,
  providerId,
  onSettingsConfigChange,
  getSettingsConfig,
}: UsePiFormStateParams): PiFormState {
  const { data: piProvidersData } = useProvidersQuery("pi");
  const existingPiKeys = useMemo(() => {
    if (!piProvidersData?.providers) return [];
    return Object.keys(piProvidersData.providers).filter(
      (k) => k !== providerId,
    );
  }, [piProvidersData?.providers, providerId]);

  const [piProviderKey, setPiProviderKey] = useState<string>(() => {
    if (appId !== "pi") return "";
    return providerId || "";
  });

  const [piBaseUrl, setPiBaseUrl] = useState<string>(() => {
    if (appId !== "pi") return "";
    return parsePiField(initialData, "baseUrl", "");
  });

  const [piApiKey, setPiApiKey] = useState<string>(() => {
    if (appId !== "pi") return "";
    return parsePiField(initialData, "apiKey", "");
  });

  const [piApi, setPiApi] = useState<PiApiMode>(() => {
    if (appId !== "pi") return PI_AGENT_DEFAULT_API_MODE;
    const stored = parsePiField<PiApiMode | "">(
      initialData,
      "api",
      "",
    );
    return stored || PI_AGENT_DEFAULT_API_MODE;
  });

  const [piModels, setPiModels] = useState<PiModel[]>(() => {
    if (appId !== "pi") return [];
    return parsePiField<PiModel[]>(initialData, "models", []);
  });

  const updatePiConfig = useCallback(
    (updater: (config: Record<string, unknown>) => void) => {
      try {
        const config = JSON.parse(
          getSettingsConfig() || PI_AGENT_DEFAULT_CONFIG,
        );
        updater(config);
        onSettingsConfigChange(JSON.stringify(config, null, 2));
      } catch {
        // ignore
      }
    },
    [getSettingsConfig, onSettingsConfigChange],
  );

  const handlePiBaseUrlChange = useCallback(
    (baseUrl: string) => {
      setPiBaseUrl(baseUrl);
      updatePiConfig((config) => {
        config.baseUrl = baseUrl.trim().replace(/\/+$/, "");
      });
    },
    [updatePiConfig],
  );

  const handlePiApiKeyChange = useCallback(
    (apiKey: string) => {
      setPiApiKey(apiKey);
      updatePiConfig((config) => {
        config.apiKey = apiKey;
      });
    },
    [updatePiConfig],
  );

  const handlePiApiChange = useCallback(
    (api: PiApiMode) => {
      setPiApi(api);
      updatePiConfig((config) => {
        config.api = api;
      });
    },
    [updatePiConfig],
  );

  const handlePiModelsChange = useCallback(
    (models: PiModel[]) => {
      setPiModels(models);
      updatePiConfig((config) => {
        if (models.length === 0) {
          delete config.models;
        } else {
          config.models = models;
        }
      });
    },
    [updatePiConfig],
  );

  const resetPiState = useCallback(
    (config?: Partial<PiProviderSettingsConfig>) => {
      setPiProviderKey("");
      setPiBaseUrl(config?.baseUrl || "");
      setPiApiKey(config?.apiKey || "");
      setPiApi(config?.api ?? PI_AGENT_DEFAULT_API_MODE);
      setPiModels(config?.models ?? []);
    },
    [],
  );

  return {
    piProviderKey,
    setPiProviderKey,
    piBaseUrl,
    piApiKey,
    piApi,
    piModels,
    existingPiKeys,
    handlePiBaseUrlChange,
    handlePiApiKeyChange,
    handlePiApiChange,
    handlePiModelsChange,
    resetPiState,
  };
}
