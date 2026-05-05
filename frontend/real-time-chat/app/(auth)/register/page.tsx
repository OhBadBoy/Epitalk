"use client"

import Link from "next/link";
import { RegisterForm } from "@/components/forms/register-form";
import { useLanguage } from "@/components/language-provider";
import { getAuthHomeLabel } from "@/lib/settings-i18n";

export default function Page() {
  const { language } = useLanguage();

  return (
    <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
      <div className="w-full max-w-sm">
        <Link
          href="/"
          className="mb-4 inline-flex h-9 items-center justify-center rounded-md border border-[#023BFC] bg-[#023BFC] px-4 py-2 text-sm font-medium text-white shadow-xs transition-colors hover:bg-[#0B33C9]"
        >
          {getAuthHomeLabel(language)}
        </Link>
        <RegisterForm />
      </div>
    </div>
  );
}
