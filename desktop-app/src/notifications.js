const { getTranslations } = require("./i18n");

function getReadyNotificationOptions(appName = "EpiTalk", lang) {
  const t = getTranslations(lang);
  return {
    title: t.readyTitle(appName),
    body: t.readyBody,
  };
}

function shouldShowReadyNotification({ isFirstLaunch, hasFocus }) {
  if (!isFirstLaunch) return false;
  if (hasFocus) return false;
  return true;
}

/**
 * Get notification options for a new server message
 * @param {string} username - The username of the message author
 * @param {string} serverName - The name of the server
 * @param {string} channelName - The name of the channel
 * @param {string} messageContent - The content of the message (will be truncated if too long)
 * @returns {Object} Notification options for Notification API
 */
function getServerMessageNotificationOptions(
  username,
  serverName,
  channelName,
  messageContent
) {
  // Truncate message if too long
  const truncatedContent =
    messageContent.length > 100
      ? messageContent.substring(0, 97) + "..."
      : messageContent;

  return {
    title: `${username} in ${serverName}`,
    body: `#${channelName}: ${truncatedContent}`,
    icon: "/favicon.ico",
    badge: "/favicon.ico",
    tag: `server-msg-${serverName}-${channelName}`, // Group similar notifications
    requireInteraction: false, // Auto-dismiss after a few seconds
  };
}

/**
 * Should show a server message notification based on app state
 * Don't show if app is focused and user is looking at that channel
 * @param {Object} config - Configuration object
 * @param {boolean} config.hasFocus - Whether the app window has focus
 * @param {string} config.currentChannelId - The channel ID currently being viewed
 * @param {string} config.messageChannelId - The channel ID where the message arrived
 * @returns {boolean} Whether to show the notification
 */
function shouldShowServerMessageNotification({
  hasFocus,
  currentChannelId,
  messageChannelId,
}) {
  // Always show if app doesn't have focus
  if (!hasFocus) {
    return true;
  }

  // If app has focus but user is NOT looking at this channel, show notification
  if (currentChannelId !== messageChannelId) {
    return true;
  }

  // Don't show if user is already looking at the channel
  return false;
}

module.exports = {
  getReadyNotificationOptions,
  shouldShowReadyNotification,
  getServerMessageNotificationOptions,
  shouldShowServerMessageNotification,
};
