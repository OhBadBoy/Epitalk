"use client";

import React from "react";
import { Toaster } from "sonner";
import { LanguageProvider } from "@/components/language-provider";

export function Providers({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <LanguageProvider>
      {children}
      <Toaster position="top-right" richColors />
    </LanguageProvider>
  );
}
