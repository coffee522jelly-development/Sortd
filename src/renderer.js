const { invoke } = window.__TAURI__.core;
const { isPermissionGranted, requestPermission, sendNotification } = window.__TAURI__.notification;

const deleteBtn = document.getElementById('deleteBtn');
const organizeBtn = document.getElementById('organizeBtn');
const classifyBtn = document.getElementById('classifyBtn');
const statusDiv = document.getElementById('status');

async function checkNotificationPermission() {
  let permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === 'granted';
  }
  return permissionGranted;
}

deleteBtn.addEventListener('click', async () => {
  statusDiv.textContent = 'ステータス: 削除中...';
  try {
    const result = await invoke('delete_new_folders');
    statusDiv.textContent = `ステータス: ${result}`;
    if (await checkNotificationPermission()) {
      sendNotification({ title: 'Desktop Organizer', body: result });
    }
  } catch (error) {
    statusDiv.textContent = `エラー: ${error}`;
  }
});

classifyBtn.addEventListener('click', async () => {
  statusDiv.textContent = 'ステータス: 分類中...';
  try {
    const result = await invoke('classify_folders');
    statusDiv.textContent = `ステータス: ${result}`;
    if (await checkNotificationPermission()) {
      sendNotification({ title: 'Desktop Organizer', body: result });
    }
  } catch (error) {
    statusDiv.textContent = `エラー: ${error}`;
  }
});

organizeBtn.addEventListener('click', async () => {
  statusDiv.textContent = 'ステータス: 整理中...';
  try {
    const result = await invoke('organize_files');
    statusDiv.textContent = `ステータス: ${result}`;
    if (await checkNotificationPermission()) {
      sendNotification({ title: 'Desktop Organizer', body: result });
    }
  } catch (error) {
    statusDiv.textContent = `エラー: ${error}`;
  }
});
