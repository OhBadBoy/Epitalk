"use client";

import { useEffect, useMemo, useState } from "react";

type HealthPayload = {
  status?: string;
  service?: string;
  version?: string;
};

type HealthState =
  | { kind: "loading" }
  | { kind: "ok"; payload: HealthPayload; checkedAt: string }
  | { kind: "error"; message: string; checkedAt: string };

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

export function BackendHealthIndicator() {
  const [state, setState] = useState<HealthState>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;

    const checkHealth = async () => {
      try {
        const response = await fetch(`${API_BASE_URL}/api/health`, {
          method: "GET",
          cache: "no-store",
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        const payload = (await response.json()) as HealthPayload;

        if (!cancelled) {
          const status = payload.status?.toLowerCase();
          if (status === "ok") {
            setState({
              kind: "ok",
              payload,
              checkedAt: new Date().toISOString(),
            });
          } else {
            setState({
              kind: "error",
              message: "Statut backend inattendu",
              checkedAt: new Date().toISOString(),
            });
          }
        }
      } catch (error) {
        if (!cancelled) {
          setState({
            kind: "error",
            message: error instanceof Error ? error.message : "Backend indisponible",
            checkedAt: new Date().toISOString(),
          });
        }
      }
    };

    void checkHealth();
    const interval = window.setInterval(() => {
      void checkHealth();
    }, 30000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, []);

  const ui = useMemo(() => {
    if (state.kind === "loading") {
      return {
        dot: "bg-amber-400",
        text: "Backend: verification...",
        title: "Verification du backend en cours",
      };
    }

    if (state.kind === "ok") {
      const suffix = state.payload.version ? ` v${state.payload.version}` : "";
      return {
        dot: "bg-emerald-500",
        text: `Backend: en ligne${suffix}`,
        title: `Dernier check ${new Date(state.checkedAt).toLocaleTimeString()}`,
      };
    }

    return {
      dot: "bg-red-500",
      text: "Backend: indisponible",
      title: `${state.message} (dernier check ${new Date(state.checkedAt).toLocaleTimeString()})`,
    };
  }, [state]);

  return (
    <div
      className="inline-flex items-center gap-2 rounded-full border border-[#E5E7EB] bg-white/85 px-3 py-1 text-[11px] font-medium text-[#374151] shadow-sm"
      title={ui.title}
    >
      <span className={`h-2 w-2 rounded-full ${ui.dot}`} />
      <span>{ui.text}</span>
    </div>
  );
}
