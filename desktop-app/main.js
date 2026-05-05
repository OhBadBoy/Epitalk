const path = require("node:path");
const { app, BrowserWindow, Menu, Notification, ipcMain } = require("electron");
const { getWebAppUrl } = require("./src/config");
const { buildMenuTemplate } = require("./src/menu");
const { detectLanguage } = require("./src/i18n");
const {
  getReadyNotificationOptions,
  shouldShowReadyNotification,
  getServerMessageNotificationOptions,
  shouldShowServerMessageNotification,
} = require("./src/notifications");

let mainWindow;

function createMainWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    icon: path.join(__dirname, "assets", "icon.svg"),
    webPreferences: {
      contextIsolation: true,
    },
  });

  const url = getWebAppUrl();
  win.loadURL(url);
  return win;
}

app.whenReady().then(() => {
  mainWindow = createMainWindow();

  const menu = Menu.buildFromTemplate(buildMenuTemplate(app.getLocale()));
  Menu.setApplicationMenu(menu);

  mainWindow.webContents.once("did-finish-load", () => {
    if (!Notification.isSupported()) return;

    const shouldNotify = shouldShowReadyNotification({
      isFirstLaunch: true,
      hasFocus: mainWindow.isFocused(),
    });

    if (!shouldNotify) return;

    const opts = getReadyNotificationOptions(app.getName(), detectLanguage());
    new Notification(opts).show();
  });

  // Listen for server message notifications from frontend
  mainWindow.webContents.on("before-input-event", (event, input) => {
    // Placeholder for IPC-style notification handling
    // This allows frontend to communicate notifications to Electron
  });

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      mainWindow = createMainWindow();
    }
  });
});

// IPC handler for server message notifications from frontend
ipcMain.handle("notify:server-message", async (event, data) => {
  if (!mainWindow || !Notification.isSupported()) {
    return false;
  }

  const shouldNotify = shouldShowServerMessageNotification({
    hasFocus: mainWindow.isFocused(),
    currentChannelId: data.currentChannelId,
    messageChannelId: data.channelId,
  });

  if (!shouldNotify) {
    return false;
  }

  try {
    const opts = getServerMessageNotificationOptions(
      data.username || "Someone",
      data.serverName || "Server",
      data.channelName || "channel",
      data.content || ""
    );
    
    const notification = new Notification(opts);
    
    // When user clicks the notification, focus the window
    notification.on("click", () => {
      if (mainWindow) {
        if (mainWindow.isMinimized()) {
          mainWindow.restore();
        }
        mainWindow.focus();
      }
    });
    
    notification.show();
    return true;
  } catch (error) {
    console.error("Failed to show notification:", error);
    return false;
  }
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

module.exports = {
  createMainWindow,
};
