"use client";

import { useState, useEffect } from "react";
import { createPortal } from "react-dom";
import { Hash, Loader2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { channelsApi } from "@/lib/api";
import { getErrorMessage } from "@/lib/api/errors";
import { useLanguage } from "@/components/language-provider";

interface CreateChannelModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  serverId: string | null;
  onSuccess: () => Promise<void>;
}

export function CreateChannelModal({
  open,
  onOpenChange,
  serverId,
  onSuccess,
}: CreateChannelModalProps) {
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { language } = useLanguage();
  const isEnglish = language === "en";

  const handleSubmit = async () => {
    if (!serverId) {
      setError(isEnglish ? "No server selected" : "Aucun serveur sélectionné");
      return;
    }

    const trimmed = name.trim();
    if (!trimmed) {
      setError(isEnglish ? "Channel name is required" : "Le nom du channel est requis");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await channelsApi.create(serverId, trimmed);
      await onSuccess();
      setName("");
      onOpenChange(false);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setName("");
      setError(null);
      onOpenChange(false);
    }
  };

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") handleClose();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, loading]);

  if (!open) return null;

  return createPortal(
    <>
      {/* Overlay */}
      <div className="fixed inset-0 z-50 bg-black/50" onClick={handleClose} />

      {/* Modal */}
      <div className="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md px-4">
        <div className="bg-[var(--card)] backdrop-blur-xl shadow-2xl rounded-2xl overflow-hidden">
          {/* Decorative header bar */}
          <div className="h-1.5 bg-gradient-to-r from-[#023BFC] via-[#3D6AFF] to-[#023BFC]" />

          <div className="p-6">
            {/* Header */}
            <div className="flex items-start justify-between mb-6">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-3 text-xl font-semibold text-[var(--foreground)]">
                  <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] flex items-center justify-center shadow-lg shrink-0">
                    <Hash className="h-5 w-5 text-white" />
                  </div>
                  {isEnglish ? "Create a channel" : "Créer un channel"}
                </div>
                <p className="text-[var(--muted-foreground)] mt-2 text-sm">
                  {isEnglish
                    ? "Channels are discussion spaces within your server."
                    : "Les channels sont des espaces de discussion au sein de votre serveur."}
                </p>
              </div>
              <button
                type="button"
                onClick={handleClose}
                disabled={loading}
                className="ml-3 mt-0.5 text-[var(--muted-foreground)] hover:text-[var(--foreground)] transition-colors shrink-0"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            {/* Fields */}
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="channel-name" className="text-sm font-medium text-[var(--foreground)]">
                  {isEnglish ? "Channel name" : "Nom du channel"}
                </Label>
                <div className="flex items-center gap-2">
                  <span className="text-[#023BFC] font-bold text-lg">#</span>
                  <Input
                    id="channel-name"
                    value={name}
                    onChange={(e) =>
                      setName(e.target.value.toLowerCase().replace(/\s+/g, "-"))
                    }
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleSubmit();
                    }}
                    placeholder={isEnglish ? "general" : "général"}
                    disabled={loading}
                    autoFocus
                    className="flex-1 h-12 px-4 rounded-xl border-[var(--border)] focus:border-[#023BFC] focus:ring-[#023BFC]/20 transition-all duration-200 bg-[var(--surface)]"
                  />
                </div>
                <p className="text-xs text-[var(--muted-foreground)]">
                  {isEnglish
                    ? "Use lowercase letters and dashes"
                    : "Utilisez des lettres minuscules et des tirets"}
                </p>
              </div>

              {error && (
                <div className="flex items-center gap-2 p-3 rounded-xl bg-red-50 border border-red-200">
                  <div className="w-5 h-5 rounded-full bg-red-100 flex items-center justify-center shrink-0">
                    <span className="text-red-500 text-xs">!</span>
                  </div>
                  <p className="text-sm text-red-600">{error}</p>
                </div>
              )}
            </div>

            {/* Footer */}
            <div className="mt-6 flex gap-3 justify-end">
              <Button
                type="button"
                variant="outline"
                onClick={handleClose}
                disabled={loading}
                className="h-11 px-6 rounded-xl border-[var(--border)] hover:bg-[var(--surface)] transition-all duration-200"
              >
                {isEnglish ? "Cancel" : "Annuler"}
              </Button>
              <Button
                type="button"
                onClick={handleSubmit}
                disabled={loading || !name.trim() || !serverId}
                className="h-11 px-6 rounded-xl bg-gradient-to-r from-[#023BFC] to-[#3D6AFF] hover:from-[#0235E0] hover:to-[#3560E8] text-white shadow-lg shadow-[#023BFC]/25 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {isEnglish ? "Creating..." : "Création..."}
                  </>
                ) : isEnglish ? (
                  "Create channel"
                ) : (
                  "Créer le channel"
                )}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </>,
    document.body
  );
}
