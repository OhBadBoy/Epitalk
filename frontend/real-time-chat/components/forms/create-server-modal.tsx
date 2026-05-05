"use client";

import { useState } from "react";
import { Plus, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { serversApi } from "@/lib/api";
import { getErrorMessage } from "@/lib/api/errors";
import { useLanguage } from "@/components/language-provider";

interface CreateServerModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => Promise<void>;
}

export function CreateServerModal({ open, onOpenChange, onSuccess }: CreateServerModalProps) {
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { language } = useLanguage();
  const isEnglish = language === "en";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const trimmed = name.trim();
    if (!trimmed) {
      setError(
        isEnglish ? "Server name is required" : "Le nom du serveur est requis",
      );
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await serversApi.create(trimmed);
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

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[425px] border-0 bg-[var(--card)] backdrop-blur-xl shadow-2xl rounded-2xl overflow-hidden">
        {/* Decorative gradient header */}
        <div className="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-[#023BFC] via-[#3D6AFF] to-[#023BFC]" />
        
        <form onSubmit={handleSubmit}>
          <DialogHeader className="pt-6">
            <DialogTitle className="flex items-center gap-3 text-xl">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] flex items-center justify-center shadow-lg">
                <Plus className="h-5 w-5 text-white" />
              </div>
              <span className="text-[var(--foreground)]">
                {isEnglish ? "Create a server" : "Créer un serveur"}
              </span>
            </DialogTitle>
            <DialogDescription className="text-[var(--muted-foreground)] mt-2">
              {isEnglish
                ? "Give your server a name. You can change it later."
                : "Donnez un nom à votre serveur. Vous pourrez le modifier plus tard."}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-6">
            <div className="grid gap-2">
              <Label htmlFor="server-name" className="text-[var(--muted-foreground)] font-medium">
                {isEnglish ? "Server name" : "Nom du serveur"}
              </Label>
              <Input
                id="server-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Mon super serveur"
                disabled={loading}
                autoFocus
                className="h-12 px-4 rounded-xl border-[var(--border)] focus:border-[#023BFC] focus:ring-2 focus:ring-[#023BFC]/20 transition-all duration-300 bg-[var(--surface)]"
              />
            </div>

            {error && (
              <div className="flex items-center gap-2 text-sm text-red-600 bg-red-50 border border-red-200 rounded-xl px-4 py-3">
                <svg className="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                {error}
              </div>
            )}
          </div>

          <DialogFooter className="gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={handleClose}
              disabled={loading}
              className="h-11 px-6 rounded-xl border-[var(--border)] text-[var(--muted-foreground)] hover:bg-[var(--surface)] hover:border-[#D1D5DB] transition-all duration-300"
            >
              {isEnglish ? "Cancel" : "Annuler"}
            </Button>
            <Button 
              type="submit" 
              disabled={loading || !name.trim()}
              className="h-11 px-6 rounded-xl bg-gradient-to-r from-[#023BFC] to-[#3D6AFF] text-white shadow-lg hover:shadow-xl transition-all duration-300 disabled:opacity-50"
            >
              {loading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {isEnglish ? "Creating..." : "Création..."}
                </>
              ) : (
                isEnglish ? "Create" : "Créer"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
