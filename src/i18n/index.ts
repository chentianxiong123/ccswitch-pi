import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zh from "./locales/zh.json";

const resources = {
  zh: { translation: zh },
} as const;

export type Language = "zh";

export function getInitialLanguage(): Language {
  if (typeof window === "undefined") return "zh";
  const stored = localStorage.getItem("language");
  if (stored === "zh") return "zh";
  return "zh";
}

void i18n.use(initReactI18next).init({
  resources,
  lng: getInitialLanguage(),
  fallbackLng: "zh",
  interpolation: { escapeValue: false },
});

export const i18nReady = new Promise<void>((resolve) => {
  i18n.on("initialized", () => resolve());
});

export default i18n;