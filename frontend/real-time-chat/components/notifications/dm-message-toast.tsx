"use client";

import { X, Reply } from "lucide-react";

interface DMMessageToastProps {
  username: string;
  message: string;
  onDismiss?: () => void;
  onView?: () => void;
}

export function DMMessageToast({
  username,
  message,
  onDismiss,
  onView,
}: DMMessageToastProps) {
  // Get first letter for avatar
  const avatarLetter = username.charAt(0).toUpperCase();

  return (
    <div className="w-full max-w-sm bg-gradient-to-r from-blue-50 to-cyan-50 dark:from-blue-950 dark:to-cyan-950 rounded-lg shadow-lg border border-blue-200 dark:border-blue-700 p-4 flex flex-col gap-3">
      {/* Header with avatar and username */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3 flex-1">
          {/* Avatar */}
          <div className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-400 to-cyan-500 flex items-center justify-center flex-shrink-0">
            <span className="text-white font-bold text-sm">{avatarLetter}</span>
          </div>
          
          {/* Username */}
          <div className="flex-1">
            <div className="text-sm font-semibold text-blue-700 dark:text-blue-300">
              {username}
            </div>
            <div className="text-xs text-blue-500 dark:text-blue-400">Message privé</div>
          </div>
        </div>

        {/* Dismiss button */}
        <button
          onClick={onDismiss}
          className="p-1 hover:bg-blue-100 dark:hover:bg-blue-800 rounded-md transition-colors flex-shrink-0"
          aria-label="Fermer"
        >
          <X className="w-4 h-4 text-blue-600 dark:text-blue-400" />
        </button>
      </div>

      {/* Message preview */}
      <div className="px-0">
        <p className="text-sm text-zinc-700 dark:text-zinc-300 line-clamp-2 break-words">
          {message}
        </p>
      </div>

      {/* Action button */}
      <div className="flex gap-2 pt-1">
        <button
          onClick={onView}
          className="flex-1 flex items-center justify-center gap-2 bg-blue-600 hover:bg-blue-700 dark:bg-blue-600 dark:hover:bg-blue-700 text-white px-3 py-2 rounded-md text-sm font-medium transition-colors"
        >
          <Reply className="w-4 h-4" />
          Répondre
        </button>
      </div>
    </div>
  );
}
