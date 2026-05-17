const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');

function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  });

  win.loadFile('index.html');
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// Placeholders for IPC handlers
ipcMain.handle('delete-new-folders', async () => {
  try {
    const desktopPath = app.getPath('desktop');
    const items = fs.readdirSync(desktopPath);
    let deletedCount = 0;

    for (const item of items) {
      if (item.includes('新しいフォルダー')) {
        const fullPath = path.join(desktopPath, item);
        const stats = fs.statSync(fullPath);
        if (stats.isDirectory()) {
          fs.rmSync(fullPath, { recursive: true, force: true });
          deletedCount++;
        }
      }
    }
    return { success: true, message: `${deletedCount}個のフォルダを削除しました。` };
  } catch (error) {
    return { success: false, message: `削除中にエラーが発生しました: ${error.message}` };
  }
});

ipcMain.handle('organize-files', async () => {
  try {
    const desktopPath = app.getPath('desktop');
    const items = fs.readdirSync(desktopPath);
    let movedCount = 0;

    for (const item of items) {
      const fullPath = path.join(desktopPath, item);
      const stats = fs.statSync(fullPath);

      if (stats.isFile()) {
        const ext = path.extname(item).toLowerCase();

        // Skip shortcuts
        if (ext === '.lnk' || ext === '.url') {
          continue;
        }

        const folderName = ext ? `◇${ext.slice(1).toUpperCase()}` : '◇NO_EXTENSION';
        const targetFolder = path.join(desktopPath, folderName);

        if (!fs.existsSync(targetFolder)) {
          fs.mkdirSync(targetFolder);
        }

        const targetPath = path.join(targetFolder, item);

        // Handle name collision
        let finalTargetPath = targetPath;
        if (fs.existsSync(targetPath)) {
          const name = path.parse(item).name;
          finalTargetPath = path.join(targetFolder, `${name}_${Date.now()}${ext}`);
        }

        fs.renameSync(fullPath, finalTargetPath);
        movedCount++;
      }
    }
    return { success: true, message: `${movedCount}個のファイルを整理しました。` };
  } catch (error) {
    return { success: false, message: `整理中にエラーが発生しました: ${error.message}` };
  }
});
