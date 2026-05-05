"use client";

import React, { createContext, useContext, useEffect, useState } from "react";
import {
  DEFAULT_LANGUAGE,
  getTranslations,
  type Language,
  type Translations,
} from "@/lib/i18n";

const STORAGE_KEY = "epitalk_language";

type LanguageContextValue = {
  language: Language;
  setLanguage: (language: Language) => void;
  translations: Translations;
};

const LanguageContext = createContext<LanguageContextValue | undefined>(
  undefined,
);

export function LanguageProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  // Important: pour éviter les erreurs d'hydratation, on ne lit pas
  // window/localStorage dans l'état initial. On commence toujours avec
  // DEFAULT_LANGUAGE, identique côté serveur et client, puis on ajuste
  // la langue dans un useEffect uniquement côté client.
  const [language, setLanguageState] = useState<Language>(DEFAULT_LANGUAGE);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "fr" || stored === "en") {
      setLanguageState(stored);
      return;
    }

    const browserLang = window.navigator.language.slice(0, 2);
    if (browserLang === "fr" || browserLang === "en") {
      setLanguageState(browserLang);
      return;
    }
  }, []);

  const setLanguage = (next: Language) => {
    setLanguageState(next);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(STORAGE_KEY, next);
    }
  };

  const value: LanguageContextValue = {
    language,
    setLanguage,
    translations: getTranslations(language),
  };

  return (
    <LanguageContext.Provider value={value}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error("useLanguage must be used within a LanguageProvider");
  }
  return context;
}
