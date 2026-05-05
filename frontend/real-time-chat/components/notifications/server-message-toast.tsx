"use client";

import { X } from "lucide-react";

interface ServerMessageToastProps {
  serverName: string;
  channelName: string;
  username: string;
  message: string;
  onDismiss?: () => void;
  onView?: () => void;
}

export function ServerMessageToast({
  serverName,
  channelName,
  username,
  message,
  onDismiss,
  onView,
}: ServerMessageToastProps) {
  return (
    <div className="w-full max-w-sm bg-gradient-to-r from-indigo-50 to-blue-50 dark:from-indigo-950 dark:to-blue-950 rounded-lg shadow-lg border border-indigo-200 dark:border-indigo-700 p-4 flex flex-col gap-3">
      {/* Header with server and channel info */}
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="text-xs font-semibold text-indigo-600 dark:text-indigo-400 uppercase tracking-wide">
            {serverName}
          </div>
          <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mt-1">
            #{channelName}
          </div>
        </div>
        <button
          onClick={onDismiss}
          className="text-zinc-500 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      {/* Author info */}
      <div className="flex items-center gap-2">
        <div className="w-7 h-7 rounded-full bg-gradient-to-br from-indigo-400 to-blue-400 flex items-center justify-center text-xs font-semibold text-white">
          {username.slice(0, 2).toUpperCase()}
        </div>
        <span className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
          {username}
        </span>
      </div>

      {/* Message preview */}
      <div className="bg-white dark:bg-zinc-800 rounded p-3 border border-zinc-200 dark:border-zinc-700">
        <p className="text-sm text-zinc-700 dark:text-zinc-300 line-clamp-2">
          {message}
        </p>
      </div>

      {/* Action button */}
      <button
        onClick={onView}
        className="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-3 rounded transition text-sm"
      >
        Voir le message
      </button>
    </div>
  );
}
