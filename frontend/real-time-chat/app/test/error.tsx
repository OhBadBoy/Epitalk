"use client";

import { useEffect } from "react";

export default function Error({ error, reset }: { error: Error; reset: () => void }) {
  useEffect(() => {
    console.error("Erreur sur /test:", error);
  }, [error]);

  return (
    <div style={{ padding: 20 }}>
      <h1>Erreur sur la page /test ❌</h1>
      <p>{error.message}</p>
      <button onClick={reset}>Réessayer</button>
    </div>
  );
}
