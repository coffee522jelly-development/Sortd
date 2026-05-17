const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  deleteNewFolders: () => ipcRenderer.invoke('delete-new-folders'),
  organizeFiles: () => ipcRenderer.invoke('organize-files'),
  classifyFolders: () => ipcRenderer.invoke('classify-folders'),
});
