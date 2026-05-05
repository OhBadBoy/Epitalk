import type { Metadata } from "next";

import "./globals.css";
import Head from "next/head";
import { Providers } from "./providers";
import { ThemeInitializer } from "./theme-initializer";

export const metadata: Metadata = {
  title: "Real Time Chat",
  description: "EpiTalk",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="fr" suppressHydrationWarning>
      <Head>
        <link
          rel="stylesheet"
          href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.13.1/font/bootstrap-icons.min.css"
        />
      </Head>
      <body>
        <ThemeInitializer />
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
