<script>
  import { invoke } from "@tauri-apps/api/core";
  import { sendNotification } from "@tauri-apps/plugin-notification";
  import { ask, message, open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { persisted } from "svelte-persisted-store";

  // Persistent settings
  let prefix = persisted("sortd_prefix", "◇");
  let todayPrefix = persisted("sortd_today_prefix", "▶");
  let excludedPaths = persisted("sortd_excluded_paths", []);
  let theme = persisted("sortd_theme", "theme-slate");

  // UI state
  let showSettings = false;
  let showTodayModal = false;
  let todayFolderName = "";
  let status = "待機中";

  const themes = [
    "theme-slate", "theme-rose", "theme-blue", "theme-green",
    "theme-orange", "theme-purple", "theme-amber", "theme-emerald",
    "theme-cyan", "theme-indigo", "theme-violet", "theme-pink",
    "theme-red", "theme-teal", "theme-sky", "theme-lime"
  ];

  onMount(() => {
    const updateTheme = () => {
      const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      document.documentElement.classList.toggle("dark", isDark);
    };
    updateTheme();
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", updateTheme);
  });

  function formatSize(bytes) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

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

  async function handleOrganizeToday() {
    if (!todayFolderName.trim()) return;
    try {
      status = "今日のファイルを整理中...";
      const count = await invoke("organize_today_files", {
        todayPrefix: $todayPrefix,
        folderName: todayFolderName.trim(),
        excluded: $excludedPaths
      });
      showTodayModal = false;
      todayFolderName = "";
      status = `${count} 個のファイルを移動しました。`;
      await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個の「今日」のファイルを整理しました。` });
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleEmptyRecycleBin() {
    if (!await ask("ごみ箱を完全に空にしますか？", { title: "Sortd", kind: "warning" })) return;
    try {
      status = "ごみ箱を空にしています...";
      const bytes = await invoke("empty_recycle_bin");
      const humanSize = formatSize(bytes);
      status = `ごみ箱を空しました (${humanSize})。`;
      await sendNotification({ title: "Sortd", body: `整理完了: ごみ箱を空にしました (合計 ${humanSize})。` });
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

<div class={$theme}>
  <main class="min-h-screen bg-slate-50 dark:bg-[#09090b] flex items-center justify-center p-4 text-slate-950 dark:text-slate-50 font-sans selection:bg-primary/20 text-xs relative">
    <div class="max-w-[280px] w-full bg-white dark:bg-[#09090b] rounded-xl shadow-sm overflow-hidden border border-slate-200 dark:border-slate-800 transition-all">
      <div class="p-6 space-y-6">

        {#if !showSettings}
          <!-- Main UI -->
          <div class="flex items-center justify-between">
            <h1 class="text-xs font-semibold tracking-tight text-slate-900 dark:text-slate-50 uppercase">Sortd</h1>
            <button on:click={() => showSettings = true} class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-slate-100 dark:hover:bg-slate-800 h-8 w-8 text-slate-500 hover:text-slate-900 dark:hover:text-slate-50">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
            </button>
          </div>

          <div class="space-y-4">
            <!-- Desktop Organization Group -->
            <div class="grid gap-3">
              <button
                on:click={handleDelete}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-rose-50 dark:hover:bg-rose-950/20 hover:text-rose-600 dark:hover:text-rose-400 px-4 py-2"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
                <span>一括削除</span>
                <span class="ml-auto text-[10px] font-normal text-slate-500 dark:text-slate-400">新しいフォルダー</span>
              </button>

              <button
                on:click={handleOrganize}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary-hover px-4 py-2"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-90" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/></svg>
                <span>拡張子ごとに整理</span>
              </button>

              <button
                on:click={handleClassify}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 px-4 py-2"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
                <span>内容で分類</span>
              </button>

              <button
                on:click={() => showTodayModal = true}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-amber-50 dark:hover:bg-amber-950/20 hover:text-amber-600 dark:hover:text-amber-400 px-4 py-2"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
                <span>今日のファイルを整理</span>
              </button>
            </div>

            <!-- Separator -->
            <div class="relative py-2">
              <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-slate-100 dark:border-slate-800"></span></div>
              <div class="relative flex justify-center text-[9px] uppercase"><span class="bg-white dark:bg-[#09090b] px-2 text-slate-400">Other Utilities</span></div>
            </div>

            <!-- Maintenance Group -->
            <div class="grid gap-3">
              <button
                on:click={handleEmptyRecycleBin}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 px-4 py-2"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                <span>ごみ箱を空にする</span>
              </button>

              <button
                on:click={handleUndo}
                class="inline-flex items-center justify-center gap-2 mt-2 whitespace-nowrap text-[10px] font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 text-slate-400 dark:text-slate-600 hover:text-primary uppercase tracking-wider underline underline-offset-4 decoration-slate-200 dark:decoration-slate-800"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10h10a8 8 0 0 1 8 8v2"/><path d="m3 10 6 6"/><path d="m3 10 6-6"/></svg>
                元に戻す
              </button>
            </div>
          </div>

          <!-- Status Bar -->
          <div class="pt-4 border-t border-slate-100 dark:border-slate-800">
            <p class="text-[10px] text-center text-slate-500 dark:text-slate-400 font-medium">
              {status}
            </p>
          </div>
        {:else}
          <!-- Settings UI -->
          <div class="flex items-center justify-between mb-2">
            <button on:click={() => showSettings = false} class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-slate-100 dark:hover:bg-slate-800 h-8 w-8 text-slate-500">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
            </button>
            <h2 class="text-xs font-semibold uppercase tracking-tight text-slate-900 dark:text-slate-50">設定</h2>
            <div class="w-8"></div>
          </div>

          <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <label for="prefix" class="text-[9px] font-medium text-slate-500 uppercase tracking-wider">整理接頭辞</label>
                <input
                  id="prefix"
                  type="text"
                  bind:value={$prefix}
                  class="flex h-8 w-full rounded-md border border-slate-200 dark:border-slate-800 bg-transparent px-2 py-1 text-xs shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring text-center font-bold"
                />
              </div>
              <div class="space-y-2">
                <label for="todayPrefix" class="text-[9px] font-medium text-slate-500 uppercase tracking-wider">日次接頭辞</label>
                <input
                  id="todayPrefix"
                  type="text"
                  bind:value={$todayPrefix}
                  class="flex h-8 w-full rounded-md border border-slate-200 dark:border-slate-800 bg-transparent px-2 py-1 text-xs shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring text-center font-bold"
                />
              </div>
            </div>

            <div class="space-y-3">
              <label class="text-[10px] font-medium leading-none text-slate-500 uppercase tracking-wider">テーマ</label>
              <div class="grid grid-cols-8 gap-2">
                {#each themes as t}
                  <button
                    on:click={() => theme.set(t)}
                    class="w-4 h-4 rounded-full border border-slate-200 dark:border-slate-800 transition-all {t} bg-primary { $theme === t ? 'ring-2 ring-ring ring-offset-2 ring-offset-white dark:ring-offset-[#09090b] scale-110' : 'hover:scale-110' }"
                  ></button>
                {/each}
              </div>
            </div>

            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-[10px] font-medium leading-none text-slate-500 uppercase tracking-wider">除外リスト</span>
                <div class="flex gap-2">
                  <button on:click={() => addExcludedItem(false)} class="inline-flex items-center justify-center rounded-md text-[10px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 h-6 px-2">ファイル</button>
                  <button on:click={() => addExcludedItem(true)} class="inline-flex items-center justify-center rounded-md text-[10px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 h-6 px-2">フォルダ</button>
                </div>
              </div>

              <div class="rounded-md border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-[#09090b] p-3">
                <div class="max-h-[120px] overflow-y-auto space-y-1.5 min-h-[40px]">
                  {#if $excludedPaths.length === 0}
                    <p class="text-[10px] text-slate-500 italic text-center py-2">除外された項目はありません</p>
                  {/if}
                  {#each $excludedPaths as path}
                    <div class="flex items-center gap-2 group">
                      <p class="flex-1 text-[9px] truncate text-slate-600 dark:text-slate-400 font-mono" title={path}>{path}</p>
                      <button on:click={() => removeExcludedItem(path)} class="inline-flex items-center justify-center rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-rose-50 dark:hover:bg-rose-950/20 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 h-5 w-5">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                      </button>
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Today's File Organization Modal -->
    {#if showTodayModal}
      <div class="absolute inset-0 bg-slate-950/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <div class="bg-white dark:bg-[#09090b] w-full max-w-[240px] rounded-lg border border-slate-200 dark:border-slate-800 shadow-2xl p-4 space-y-4 animate-in fade-in zoom-in duration-200">
          <div class="space-y-1 text-center">
            <h3 class="text-[11px] font-bold tracking-tight">フォルダ名の入力</h3>
            <p class="text-[9px] text-slate-500">今日更新されたファイルを整理します</p>
          </div>
          <input
            type="text"
            bind:value={todayFolderName}
            placeholder="例: 会議資料"
            class="flex h-9 w-full rounded-md border border-slate-200 dark:border-slate-800 bg-transparent px-3 py-1 text-xs shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            autofocus
            on:keydown={(e) => e.key === 'Enter' && handleOrganizeToday()}
          />
          <div class="flex gap-2">
            <button
              on:click={() => { showTodayModal = false; todayFolderName = ""; }}
              class="flex-1 h-8 rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] text-[10px] font-medium hover:bg-slate-100 dark:hover:bg-slate-800"
            >
              キャンセル
            </button>
            <button
              on:click={handleOrganizeToday}
              disabled={!todayFolderName.trim()}
              class="flex-1 h-8 rounded-md bg-primary text-primary-foreground text-[10px] font-medium hover:bg-primary-hover disabled:opacity-50 shadow-sm"
            >
              実行
            </button>
          </div>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  /* Local overrides for theme dots in settings */
  .theme-slate { --theme-primary: theme('colors.slate.600'); }
  .theme-rose { --theme-primary: theme('colors.rose.600'); }
  .theme-blue { --theme-primary: theme('colors.blue.600'); }
  .theme-green { --theme-primary: theme('colors.green.600'); }
  .theme-orange { --theme-primary: theme('colors.orange.600'); }
  .theme-purple { --theme-primary: theme('colors.purple.600'); }
  .theme-amber { --theme-primary: theme('colors.amber.600'); }
  .theme-emerald { --theme-primary: theme('colors.emerald.600'); }
  .theme-cyan { --theme-primary: theme('colors.cyan.600'); }
  .theme-indigo { --theme-primary: theme('colors.indigo.600'); }
  .theme-violet { --theme-primary: theme('colors.violet.600'); }
  .theme-pink { --theme-primary: theme('colors.pink.600'); }
  .theme-red { --theme-primary: theme('colors.red.600'); }
  .theme-teal { --theme-primary: theme('colors.teal.600'); }
  .theme-sky { --theme-primary: theme('colors.sky.600'); }
  .theme-lime { --theme-primary: theme('colors.lime.600'); }

  :global(.dark) .theme-slate { --theme-primary: theme('colors.slate.400'); }
  :global(.dark) .theme-rose { --theme-primary: theme('colors.rose.400'); }
  :global(.dark) .theme-blue { --theme-primary: theme('colors.blue.400'); }
  :global(.dark) .theme-green { --theme-primary: theme('colors.green.400'); }
  :global(.dark) .theme-orange { --theme-primary: theme('colors.orange.400'); }
  :global(.dark) .theme-purple { --theme-primary: theme('colors.purple.400'); }
  :global(.dark) .theme-amber { --theme-primary: theme('colors.amber.400'); }
  :global(.dark) .theme-emerald { --theme-primary: theme('colors.emerald.400'); }
  :global(.dark) .theme-cyan { --theme-primary: theme('colors.cyan.400'); }
  :global(.dark) .theme-indigo { --theme-primary: theme('colors.indigo.400'); }
  :global(.dark) .theme-violet { --theme-primary: theme('colors.violet.400'); }
  :global(.dark) .theme-pink { --theme-primary: theme('colors.pink.400'); }
  :global(.dark) .theme-red { --theme-primary: theme('colors.red.400'); }
  :global(.dark) .theme-teal { --theme-primary: theme('colors.teal.400'); }
  :global(.dark) .theme-sky { --theme-primary: theme('colors.sky.400'); }
  :global(.dark) .theme-lime { --theme-primary: theme('colors.lime.400'); }

  .animate-in {
    animation: animate-in 0.2s ease-out;
  }
  @keyframes animate-in {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
