"use client";

import { ReactNode } from "react";

interface ProgressiveBlurProps {
  children: ReactNode;
  className?: string;
}

export function ProgressiveBlur({
  children,
  className = "",
}: ProgressiveBlurProps) {
  return (
    <div className={`relative ${className}`}>
      {children}
      <div className="pointer-events-none absolute inset-0 bg-linear-to-b from-transparent via-transparent to-background/80" />
    </div>
  );
}
