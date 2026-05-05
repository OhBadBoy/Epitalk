"use client";

import React, { useEffect, useState } from "react";
import { serversApi } from "@/lib/api";
import { ApiError } from "@/lib/api/errors";
import { terminateSession } from "@/lib/auth/session";
import type { Server } from "@/lib/api/schemas/servers.schema";
import { useServerStore } from "@/store/server.store";
import { useRouter } from "next/navigation";

export function ServersLoader({
  children,
}: {
  children: (data: {
    servers: Server[];
    refresh: () => Promise<void>;
    loading: boolean;
  }) => React.ReactNode;
}) {
  const router = useRouter();
  const [servers, setServersState] = useState<Server[]>([]);
  const [loading, setLoading] = useState(true);

  const setServers = useServerStore((s) => s.setServers);
  const activeServerId = useServerStore((s) => s.activeServerId);
  const setActiveServer = useServerStore((s) => s.setActiveServer);

  const refresh = async () => {
    try {
      const data = await serversApi.list();
      setServersState(data);
      setServers(data);

      // Auto-select si aucun serveur actif
      if (!activeServerId && data.length > 0) {
        setActiveServer(data[0].id);
      }
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        terminateSession();
        router.replace("/login");
        return;
      }
      throw error;
    }
  };

  useEffect(() => {
    (async () => {
      try {
        await refresh();
      } catch {
        setServersState([]);
        setServers([]);
      } finally {
        setLoading(false);
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return <>{children({ servers, refresh, loading })}</>;
}
