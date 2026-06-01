const { invoke } = window.__TAURI__.core;
const { sendNotification } = window.__TAURI_PLUGIN_NOTIFICATION__;
const { ask, message } = window.__TAURI_PLUGIN_DIALOG__;

const deleteBtn = document.getElementById('deleteBtn');
const organizeBtn = document.getElementById('organizeBtn');
const classifyBtn = document.getElementById('classifyBtn');
const undoBtn = document.getElementById('undoBtn');
const statusDiv = document.getElementById('status');
const prefixInput = document.getElementById('prefixInput');
const excludeInput = document.getElementById('excludeInput');

function setStatus(msg) {
  statusDiv.innerText = `ステータス: ${msg}`;
  console.log(msg);
}

function getExclusions() {
  return excludeInput.value
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0);
}

deleteBtn.addEventListener('click', async () => {
  const confirmed = await ask('「新しいフォルダー」をすべて削除しますか？', {
    title: 'Sortd',
    kind: 'warning',
  });

  if (!confirmed) return;

  try {
    const excluded = getExclusions();
    setStatus('削除中...');
    const count = await invoke('delete_new_folders', { excluded });
    setStatus(`${count} 個のフォルダを削除しました。`);
    await sendNotification({ title: 'Sortd', body: `整理完了: ${count} 個の空の「新しいフォルダー」を削除しました。` });
  } catch (err) {
    setStatus(`エラー: ${err}`);
  }
});

organizeBtn.addEventListener('click', async () => {
  const prefix = prefixInput.value || '◇';
  const excluded = getExclusions();
  try {
    setStatus('プレビューを取得中...');
    const preview = await invoke('get_organization_preview', { prefix, excluded });

    if (preview.length === 0) {
      await message('整理が必要なファイルはありません。', { title: 'Sortd', kind: 'info' });
      setStatus('整理不要');
      return;
    }

    const previewText = preview.map(p => `${p.filename} -> ${p.target_dir}`).join('\n');
    const confirmed = await ask(`以下のファイルを整理しますか？\n\n${previewText.substring(0, 500)}${previewText.length > 500 ? '...' : ''}`, {
      title: 'Sortd - プレビュー',
    });

    if (!confirmed) {
      setStatus('キャンセルされました');
      return;
    }

    setStatus('整理中...');
    const count = await invoke('organize_files', { prefix, excluded });
    setStatus(`${count} 個のファイルを整理しました。`);
    await sendNotification({ title: 'Sortd', body: `整理完了: ${count} 個のファイルを拡張子別に移動しました。` });
  } catch (err) {
    setStatus(`エラー: ${err}`);
  }
});

classifyBtn.addEventListener('click', async () => {
  const prefix = prefixInput.value || '◇';
  const excluded = getExclusions();
  try {
    setStatus('分類のプレビューを取得中...');
    const preview = await invoke('get_classification_preview', { prefix, excluded });

    if (preview.length === 0) {
      await message('分類可能なフォルダは見つかりませんでした。', { title: 'Sortd', kind: 'info' });
      setStatus('分類不要');
      return;
    }

    const previewText = preview.map(p => `${p.folder_name} -> ${p.category}`).join('\n');
    const confirmed = await ask(`以下のフォルダを分類しますか？\n\n${previewText.substring(0, 500)}${previewText.length > 500 ? '...' : ''}`, {
      title: 'Sortd - 分類プレビュー',
    });

    if (!confirmed) {
      setStatus('キャンセルされました');
      return;
    }

    setStatus('分類中...');
    const count = await invoke('classify_folders', { prefix, excluded });
    setStatus(`${count} 個のフォルダを分類しました。`);
    await sendNotification({ title: 'Sortd', body: `分類完了: ${count} 個のフォルダをプロジェクト種別ごとに整理しました。` });
  } catch (err) {
    setStatus(`エラー: ${err}`);
  }
});

undoBtn.addEventListener('click', async () => {
  try {
    setStatus('元に戻しています...');
    const count = await invoke('undo_last_operation');
    setStatus(`${count} 件の変更を元に戻しました。`);
    await sendNotification({ title: 'Sortd', body: `元に戻す完了: ${count} 件のファイルを元の場所へ戻しました。` });
  } catch (err) {
    setStatus(`エラー: ${err}`);
    await message(err, { title: 'Sortd', kind: 'error' });
  }
});
