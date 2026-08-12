import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zh from "./locales/zh.json";
import en from "./locales/en.json";

const resources = {
  zh: { translation: zh },
  en: { translation: en },
} as const;

export type Language = "zh" | "en";

export function getInitialLanguage(): Language {
  if (typeof window === "undefined") return "zh";
  const stored = localStorage.getItem("language");
  if (stored === "en") return "en";
  return "zh";
}

export const i18nReady = i18n.use(initReactI18next).init({
  resources,
  lng: getInitialLanguage(),
  fallbackLng: "zh",
  interpolation: { escapeValue: false },
});

export default i18n;
