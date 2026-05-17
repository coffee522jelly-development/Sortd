const { app, BrowserWindow, ipcMain, Menu } = require('electron');
const path = require('path');
const fs = require('fs');

function createWindow() {
  // Remove default menu
  Menu.setApplicationMenu(null);

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

ipcMain.handle('classify-folders', async () => {
  try {
    const desktopPath = app.getPath('desktop');
    const items = fs.readdirSync(desktopPath);
    let movedCount = 0;

    for (const item of items) {
      const fullPath = path.join(desktopPath, item);
      const stats = fs.statSync(fullPath);

      // We only care about directories, and skip our own "◇" category folders
      if (stats.isDirectory() && !item.startsWith('◇')) {
        const subItems = fs.readdirSync(fullPath);
        const exts = new Set();

        // Recursively or just shallow scan? Shallow scan seems enough for classification.
        for (const subItem of subItems) {
          const subFullPath = path.join(fullPath, subItem);
          const subStats = fs.statSync(subFullPath);
          if (subStats.isFile()) {
            exts.add(path.extname(subItem).toLowerCase());
          }
        }

        let category = '◇NoCategories';

        // Define extension groups
        const webExts = ['.html', '.htm', '.css', '.js', '.ts', '.jsx', '.tsx', '.php', '.vue', '.scss'];
        const unityExts = ['.unity', '.prefab', '.asset'];
        const pythonExts = ['.py', '.ipynb'];
        const designExts = ['.psd', '.ai', '.xd', '.fig', '.sketch'];
        const docExts = ['.docx', '.pptx', '.pdf', '.csv'];
        const progExts = ['.c', '.cpp', '.h', '.hpp', '.cs', '.java', '.go', '.rs', '.rb'];

        const hasExt = (list) => list.some(e => exts.has(e));

        if (hasExt(webExts)) {
          category = '◇WebProject';
        } else if (hasExt(unityExts)) {
          category = '◇UnityProject';
        } else if (hasExt(pythonExts)) {
          category = '◇PythonProject';
        } else if (hasExt(designExts)) {
          category = '◇DesignProject';
        } else if (hasExt(docExts)) {
          category = '◇DocumentProject';
        } else if (hasExt(progExts)) {
          category = '◇ProgrammingProject';
        } else if ((exts.has('.xlsx') || exts.has('.xls')) && exts.has('.png')) {
          category = '◇ProjectWorking';
        } else if (exts.has('.txt') && exts.size === 1) {
          category = '◇Memo';
        }

        const targetFolder = path.join(desktopPath, category);
        if (!fs.existsSync(targetFolder)) {
          fs.mkdirSync(targetFolder);
        }

        const targetPath = path.join(targetFolder, item);
        let finalTargetPath = targetPath;
        if (fs.existsSync(targetPath)) {
          finalTargetPath = path.join(targetFolder, `${item}_${Date.now()}`);
        }

        fs.renameSync(fullPath, finalTargetPath);
        movedCount++;
      }
    }
    return { success: true, message: `${movedCount}個のフォルダを分類しました。` };
  } catch (error) {
    return { success: false, message: `分類中にエラーが発生しました: ${error.message}` };
  }
});
