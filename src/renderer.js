const { invoke } = window.__TAURI__.core;
const { isPermissionGranted, requestPermission, sendNotification } = window.__TAURI__.notification;
const { ask, message } = window.__TAURI__.dialog;

const prefixInput = document.getElementById('prefixInput');
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
  const prefix = prefixInput.value || '◇';
  statusDiv.textContent = 'ステータス: 分類中...';
  try {
    const result = await invoke('classify_folders', { prefix });
    statusDiv.textContent = `ステータス: ${result}`;
    if (await checkNotificationPermission()) {
      sendNotification({ title: 'Desktop Organizer', body: result });
    }
  } catch (error) {
    statusDiv.textContent = `エラー: ${error}`;
  }
});

organizeBtn.addEventListener('click', async () => {
  const prefix = prefixInput.value || '◇';
  statusDiv.textContent = 'ステータス: プレビュー取得中...';

  try {
    const preview = await invoke('get_organization_preview', { prefix });

    if (preview.length === 0) {
      await message('整理するファイルが見つかりませんでした。', { title: 'Desktop Organizer', kind: 'info' });
      statusDiv.textContent = 'ステータス: 待機中';
      return;
    }

    const previewText = preview.map(([file, folder]) => `${file} -> ${folder}`).join('\n');
    const confirmed = await ask(`以下の内容で整理を実行しますか？\n\n${previewText}`, {
      title: '実行の確認',
      kind: 'info',
      okLabel: '実行',
      cancelLabel: 'キャンセル'
    });

    if (confirmed) {
      statusDiv.textContent = 'ステータス: 整理中...';
      const result = await invoke('organize_files', { prefix });
      statusDiv.textContent = `ステータス: ${result}`;
      if (await checkNotificationPermission()) {
        sendNotification({ title: 'Desktop Organizer', body: result });
      }
    } else {
      statusDiv.textContent = 'ステータス: キャンセルされました';
    }
  } catch (error) {
    statusDiv.textContent = `エラー: ${error}`;
  }
});
