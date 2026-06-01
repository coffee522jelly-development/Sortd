<script>
  import { invoke } from "@tauri-apps/api/core";
  import { sendNotification } from "@tauri-apps/plugin-notification";
  import { ask, message, open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { persisted } from "svelte-persisted-store";
  import "../app.css";

  let prefix = persisted("sortd_prefix", "◇");
  let excludedPaths = persisted("sortd_excluded_paths", []);
  let showSettings = false;
  let status = "待機中";

  onMount(() => {
    const updateTheme = () => {
      const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      document.documentElement.classList.toggle("dark", isDark);
    };
    updateTheme();
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", updateTheme);
  });

  async function addExcludedItem(directory = true) {
    try {
      const selected = await open({
        directory,
        multiple: true,
        title: directory ? "除外するフォルダを選択" : "除外するファイルを選択",
      });
      if (selected) {
        const newPaths = Array.isArray(selected) ? selected : [selected];
        excludedPaths.update(paths => [...new Set([...paths, ...newPaths])]);
      }
    } catch (err) { console.error(err); }
  }

  function removeExcludedItem(path) {
    excludedPaths.update(paths => paths.filter(p => p !== path));
  }

  async function handleDelete() {
    if (!await ask("「新しいフォルダー」をすべて削除しますか？", { title: "Sortd", kind: "warning" })) return;
    try {
      status = "削除中...";
      const count = await invoke("delete_new_folders", { excluded: $excludedPaths });
      status = `${count} 個のフォルダを削除しました。`;
      await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個の空の「新しいフォルダー」を削除しました。` });
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleOrganize() {
    try {
      status = "プレビューを取得中...";
      const preview = await invoke("get_organization_preview", { prefix: $prefix, excluded: $excludedPaths });
      if (preview.length === 0) {
        await message("整理が必要なファイルはありません。", { title: "Sortd", kind: "info" });
        status = "整理不要"; return;
      }
      const previewText = preview.map((p) => `${p.filename} -> ${p.target_dir}`).join("\n");
      if (!await ask(`以下のファイルを整理しますか？\n\n${previewText.substring(0, 500)}...`, { title: "Sortd - プレビュー" })) return;
      status = "整理中...";
      const count = await invoke("organize_files", { prefix: $prefix, excluded: $excludedPaths });
      status = `${count} 個のファイルを整理しました。`;
      await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個のファイルを拡張子別に移動しました。` });
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleClassify() {
    try {
      status = "分類のプレビューを取得中...";
      const preview = await invoke("get_classification_preview", { prefix: $prefix, excluded: $excludedPaths });
      if (preview.length === 0) {
        await message("分類可能なフォルダは見つかりませんでした。", { title: "Sortd", kind: "info" });
        status = "分類不要"; return;
      }
      const previewText = preview.map((p) => `${p.folder_name} -> ${p.category}`).join("\n");
      if (!await ask(`以下のフォルダを分類しますか？\n\n${previewText.substring(0, 500)}...`, { title: "Sortd - 分類プレビュー" })) return;
      status = "分類中...";
      const count = await invoke("classify_folders", { prefix: $prefix, excluded: $excludedPaths });
      status = `${count} 個のフォルダを分類しました。`;
      await sendNotification({ title: "Sortd", body: `分類完了: ${count} 個のフォルダをプロジェクト種別ごとに整理しました。` });
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleUndo() {
    try {
      status = "元に戻しています...";
      const count = await invoke("undo_last_operation");
      status = `${count} 件の変更を元に戻しました。`;
      await sendNotification({ title: "Sortd", body: `元に戻す完了: ${count} 件のファイルを元の場所へ戻しました。` });
    } catch (err) {
      status = `エラー: ${err}`;
      await message(err, { title: "Sortd", kind: "error" });
    }
  }
</script>

<main class="min-h-screen bg-slate-100 dark:bg-slate-950 flex items-center justify-center p-4 text-slate-900 dark:text-slate-100 font-sans selection:bg-blue-200 dark:selection:bg-blue-800">
  <div class="max-w-xs w-full bg-white dark:bg-slate-900 rounded-xl shadow-2xl overflow-hidden border border-slate-200 dark:border-slate-800 transition-all">
    <div class="p-5 space-y-4">
      {#if !showSettings}
        <div class="flex items-center justify-between">
          <h1 class="text-sm font-black tracking-tighter text-slate-400 dark:text-slate-600">SORTD</h1>
          <button on:click={() => showSettings = true} class="p-1.5 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </button>
        </div>
        <div class="grid gap-2">
          <button on:click={handleDelete} class="flex items-center gap-3 w-full px-3 py-2 bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50 rounded-lg text-xs font-semibold hover:bg-rose-100 dark:hover:bg-rose-900/50 transition-colors group">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
            一括削除 <span class="ml-auto text-[10px] opacity-50 font-normal italic">新しいフォルダー</span>
          </button>
          <button on:click={handleOrganize} class="flex items-center gap-3 w-full px-3 py-2 bg-blue-600 text-white rounded-lg text-xs font-semibold shadow-lg shadow-blue-500/20 hover:bg-blue-700 transition-colors group">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-80 group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/></svg>
            拡張子ごとに整理
          </button>
          <button on:click={handleClassify} class="flex items-center gap-3 w-full px-3 py-2 bg-slate-800 dark:bg-slate-700 text-white rounded-lg text-xs font-semibold shadow-lg shadow-slate-900/20 hover:bg-slate-900 dark:hover:bg-slate-600 transition-colors group">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-80 group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
            フォルダを内容で分類
          </button>
          <button on:click={handleUndo} class="flex items-center justify-center gap-2 mt-2 w-full py-1 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 transition-colors uppercase tracking-widest">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10h10a8 8 0 0 1 8 8v2"/><path d="m3 10 6 6"/><path d="m3 10 6-6"/></svg>
            元に戻す
          </button>
        </div>
        <div class="mt-4 px-3 py-2 bg-slate-50 dark:bg-slate-950/50 rounded-lg border border-slate-100 dark:border-slate-800">
          <p class="text-[10px] text-center text-slate-500 dark:text-slate-400 italic leading-snug">{status}</p>
        </div>
      {:else}
        <div class="flex items-center justify-between mb-4">
          <button on:click={() => showSettings = false} class="p-1.5 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
          </button>
          <h2 class="text-xs font-bold uppercase tracking-wider text-slate-500">設定</h2>
          <div class="w-7"></div>
        </div>
        <div class="space-y-4">
          <div class="flex items-center justify-between gap-4">
            <label for="prefix" class="text-xs font-bold text-slate-500 dark:text-slate-400">フォルダ接頭辞</label>
            <input id="prefix" type="text" bind:value={$prefix} class="w-10 h-7 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-center text-xs font-bold focus:ring-2 focus:ring-blue-500 outline-none transition-all" />
          </div>
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">除外リスト</span>
              <div class="flex gap-1">
                <button on:click={() => addExcludedItem(false)} class="text-[10px] bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded hover:bg-slate-200 dark:hover:bg-slate-700">ファイル追加</button>
                <button on:click={() => addExcludedItem(true)} class="text-[10px] bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded hover:bg-slate-200 dark:hover:bg-slate-700">フォルダ追加</button>
              </div>
            </div>
            <div class="max-h-40 overflow-y-auto space-y-1 bg-slate-50 dark:bg-slate-950/50 p-2 rounded-lg border border-slate-100 dark:border-slate-800 min-h-[4rem]">
              {#if $excludedPaths.length === 0}
                <p class="text-[10px] text-slate-400 italic text-center mt-4">除外項目なし</p>
              {/if}
              {#each $excludedPaths as path}
                <div class="flex items-center gap-2 group">
                  <p class="flex-1 text-[9px] truncate text-slate-600 dark:text-slate-400" title={path}>{path}</p>
                  <button on:click={() => removeExcludedItem(path)} class="text-rose-500 opacity-0 group-hover:opacity-100 hover:text-rose-600">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                  </button>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</main>
