const deleteBtn = document.getElementById('deleteBtn');
const organizeBtn = document.getElementById('organizeBtn');
const statusDiv = document.getElementById('status');

deleteBtn.addEventListener('click', async () => {
  statusDiv.textContent = 'ステータス: 削除中...';
  try {
    const result = await window.electronAPI.deleteNewFolders();
    statusDiv.textContent = `ステータス: ${result.message}`;
  } catch (error) {
    statusDiv.textContent = `エラー: ${error.message}`;
  }
});

organizeBtn.addEventListener('click', async () => {
  statusDiv.textContent = 'ステータス: 整理中...';
  try {
    const result = await window.electronAPI.organizeFiles();
    statusDiv.textContent = `ステータス: ${result.message}`;
  } catch (error) {
    statusDiv.textContent = `エラー: ${error.message}`;
  }
});
