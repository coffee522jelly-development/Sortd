<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
      import { sendNotification } from "@tauri-apps/plugin-notification";
  import { ask, message, open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { persisted } from "svelte-persisted-store";

  // Persistent settings
  let prefix = persisted("sortd_prefix", "◇");
  let todayPrefix = persisted("sortd_today_prefix", "▶");
  let excludedPaths = persisted("sortd_excluded_paths", []);
  let theme = persisted("sortd_theme", "theme-slate");
  let customColor = persisted("sortd_custom_color", "");
  let colorMode = persisted("sortd_color_mode", "system"); // "system", "light", "dark"

  const defaultRules = [
    { name: "WebProject", extensions: ["html", "htm", "css", "js", "ts", "jsx", "tsx", "php", "vue", "scss"] },
    { name: "UnityProject", extensions: ["unity", "prefab", "asset"] },
    { name: "PythonProject", extensions: ["py", "ipynb"] },
    { name: "DesignProject", extensions: ["psd", "ai", "xd", "fig", "sketch"] },
    { name: "DocumentProject", extensions: ["docx", "pptx", "pdf", "csv"] },
    { name: "ProgrammingProject", extensions: ["c", "cpp", "h", "hpp", "cs", "java", "go", "rs", "rb"] },
    { name: "ProjectWorking", extensions: ["xlsx", "xls", "png"] },
    { name: "Memo", extensions: ["txt"] }
  ];
  let customRules = persisted("sortd_custom_rules", defaultRules);

  // UI state
  let showSettings = false;
  let showTodayModal = false;
  let newRuleName = "";
  let newRuleExts = "";

  function cycleColorMode() {
    if ($colorMode === "system") colorMode.set("light");
    else if ($colorMode === "light") colorMode.set("dark");
    else colorMode.set("system");
  }

  function addCustomRule() {
    const name = newRuleName.trim();
    const exts = newRuleExts.split(',').map(e => e.trim().toLowerCase()).filter(e => e.length > 0);
    if (name && exts.length > 0) {
      customRules.update(rules => [...rules, { name, extensions: exts }]);
      newRuleName = "";
      newRuleExts = "";
    }
  }


  function updateRuleName(index, event) {
    const val = event.target.value.trim();
    if (val) {
      customRules.update(rules => {
        rules[index].name = val;
        return rules;
      });
    }
  }

  function updateRuleExts(index, event) {
    const val = event.target.value.trim();
    if (val) {
      const exts = val.split(',').map(e => e.trim().toLowerCase()).filter(e => e.length > 0);
      customRules.update(rules => {
        rules[index].extensions = exts;
        return rules;
      });
    }
  }

  function removeCustomRule(index) {
    customRules.update(rules => rules.filter((_, i) => i !== index));
  }

  function exportRules() {
    const dataStr = JSON.stringify($customRules, null, 2);
    const blob = new Blob([dataStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "sortd_rules.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function importRules(event) {
    const file = event.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const rules = JSON.parse(e.target.result);
        if (Array.isArray(rules)) {
          customRules.set(rules);
          await message("ルールをインポートしました。", { title: "Sortd", kind: "info" });
        } else {
          throw new Error("無効なフォーマットです");
        }
      } catch (err) {
        await message(`インポートに失敗しました: ${err}`, { title: "Sortd", kind: "error" });
      }
      event.target.value = ''; // Reset input
    };
    reader.readAsText(file);
  }

  let todayFolderName = "";
  let showPreviewModal = false;
  let previewTitle = "";
  let previewItems = [];
  let onPreviewConfirm = () => {};
  let status = "待機中";
  let history = [];
  function addHistory(msg) {
    const time = new Date().toLocaleTimeString('ja-JP', { hour: '2-digit', minute:'2-digit', second:'2-digit' });
    history = [{ time, msg }, ...history].slice(0, 50); // Keep last 50
  }
  let contextMenu = { show: false, x: 0, y: 0 };

  function handleContextMenu(e) {
    e.preventDefault();
    contextMenu = {
      show: true,
      x: e.clientX,
      y: e.clientY
    };
  }

  function closeContextMenu() {
    contextMenu.show = false;
  }

  async function openSettings() {
    showSettings = true;
  }

  async function closeSettings() {
    showSettings = false;
  }

  const presetColors = [
    { name: "Slate", value: "#475569", class: "theme-slate" },
    { name: "Rose", value: "#e11d48", class: "theme-rose" },
    { name: "Blue", value: "#2563eb", class: "theme-blue" },
    { name: "Green", value: "#16a34a", class: "theme-green" },
    { name: "Orange", value: "#ea580c", class: "theme-orange" },
    { name: "Purple", value: "#9333ea", class: "theme-purple" },
    { name: "Amber", value: "#d97706", class: "theme-amber" },
    { name: "Emerald", value: "#059669", class: "theme-emerald" },
    { name: "Cyan", value: "#0891b2", class: "theme-cyan" },
    { name: "Indigo", value: "#4f46e5", class: "theme-indigo" },
    { name: "Violet", value: "#7c3aed", class: "theme-violet" },
    { name: "Pink", value: "#db2777", class: "theme-pink" },
    { name: "Red", value: "#dc2626", class: "theme-red" },
    { name: "Teal", value: "#0d9488", class: "theme-teal" },
    { name: "Sky", value: "#0284c7", class: "theme-sky" },
    { name: "Lime", value: "#65a30d", class: "theme-lime" }
  ];

  let mediaQuery;

  function applyThemeMode() {
    let isDark;
    if ($colorMode === "dark") {
      isDark = true;
    } else if ($colorMode === "light") {
      isDark = false;
    } else {
      isDark = mediaQuery ? mediaQuery.matches : window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    document.documentElement.classList.toggle("dark", isDark);
  }

  $: if ($colorMode) {
    if (typeof document !== "undefined") applyThemeMode();
  }

  onMount(() => {
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleMediaChange = () => {
      if ($colorMode === "system") applyThemeMode();
    };
    applyThemeMode();
    mediaQuery.addEventListener("change", handleMediaChange);

    const unlisteners = [];

    // Tray event listeners
    const setupTrayListeners = async () => {
      unlisteners.push(await listen('tray-action-delete', () => handleDelete()));
      unlisteners.push(await listen('tray-action-organize', () => handleOrganize()));
      unlisteners.push(await listen('tray-action-classify', () => handleClassify()));
      unlisteners.push(await listen('tray-action-today', () => showTodayModal = true));
      unlisteners.push(await listen('tray-action-empty', () => handleEmptyRecycleBin()));
      unlisteners.push(await listen('tray-action-duplicates', () => handleDeleteDuplicates()));
      unlisteners.push(await listen('tray-action-settings', () => openSettings()));
    };

    let listenersReady = false;
    setupTrayListeners().then(() => listenersReady = true);

    return () => {
      mediaQuery.removeEventListener("change", handleMediaChange);
      if (listenersReady) unlisteners.forEach(unlisten => unlisten());
    };
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
      status = `${count} 個のフォルダを削除しました。`; addHistory(status);
      await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個の空の「新しいフォルダー」を削除しました。` });
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleDeleteDuplicates() {
    if (!await ask("重複ファイル（「〜のコピー」「〜 (1)」など）を削除しますか？\n※元のファイルが存在する場合のみ削除されます。", { title: "Sortd", kind: "warning" })) return;
    try {
      status = "重複ファイルを削除中...";
      const count = await invoke("delete_duplicate_files", { excluded: $excludedPaths });
      status = `${count} 個の重複ファイルを削除しました。`; addHistory(status);
      await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個の重複ファイルを削除しました。` });
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

      previewTitle = "拡張子ごとに整理";
      previewItems = preview.map(p => ({ from: p.filename, to: p.target_dir }));
      onPreviewConfirm = async () => {
        status = "整理中...";
        const count = await invoke("organize_files", { prefix: $prefix, excluded: $excludedPaths });
        status = `${count} 個のファイルを整理しました。`; addHistory(status);
        await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個のファイルを拡張子別に移動しました。` });
      };
      showPreviewModal = true;
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleClassify() {
    try {
      status = "分類のプレビューを取得中...";
      const preview = await invoke("get_classification_preview", { prefix: $prefix, excluded: $excludedPaths, rules: $customRules });
      if (preview.length === 0) {
        await message("分類可能なフォルダは見つかりませんでした。", { title: "Sortd", kind: "info" });
        status = "分類不要"; return;
      }

      previewTitle = "フォルダを分類";
      previewItems = preview.map(p => ({ from: p.folder_name, to: p.category }));
      onPreviewConfirm = async () => {
        status = "分類中...";
        const count = await invoke("classify_folders", { prefix: $prefix, excluded: $excludedPaths, rules: $customRules });
        status = `${count} 個のフォルダを分類しました。`; addHistory(status);
        await sendNotification({ title: "Sortd", body: `分類完了: ${count} 個のフォルダをプロジェクト種別ごとに整理しました。` });
      };
      showPreviewModal = true;
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
      if (count === 0) {
        status = "フォルダのみ作成しました。"; addHistory(status);
        await sendNotification({ title: "Sortd", body: "整理完了: 対象ファイルがなかったため、フォルダのみ作成しました。" });
      } else {
        status = `${count} 個のファイルを移動しました。`; addHistory(status);
        await sendNotification({ title: "Sortd", body: `整理完了: ${count} 個の「今日」のファイルを整理しました。` });
      }
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleEmptyRecycleBin() {
    if (!await ask("ごみ箱を完全に空にしますか？", { title: "Sortd", kind: "warning" })) return;
    try {
      status = "ごみ箱を空にしています...";
      const bytes = await invoke("empty_recycle_bin");
      const humanSize = formatSize(bytes);
      status = `ごみ箱を空しました (${humanSize})。`; addHistory(status);
      await sendNotification({ title: "Sortd", body: `整理完了: ごみ箱を空にしました (合計 ${humanSize})。` });
    } catch (err) { status = `エラー: ${err}`; }
  }

  async function handleUndo() {
    try {
      status = "元に戻しています...";
      const count = await invoke("undo_last_operation");
      status = `${count} 件の変更を元に戻しました。`; addHistory(status);
      await sendNotification({ title: "Sortd", body: `元に戻す完了: ${count} 件のファイルを元の場所へ戻しました。` });
    } catch (err) {
      status = `エラー: ${err}`;
      await message(err, { title: "Sortd", kind: "error" });
    }
  }
</script>

<svelte:window on:click={closeContextMenu} />
<div class={$theme} style={$customColor ? `--theme-primary: ${$customColor}; --theme-primary-hover: ${$customColor}dd;` : ""}>
  <main on:contextmenu={handleContextMenu} class="h-screen w-screen no-scrollbar overflow-hidden bg-slate-50 dark:bg-[#09090b] flex items-center justify-center p-4 text-slate-950 dark:text-slate-50 font-sans selection:bg-primary/20 text-xs relative">
    <div class="max-w-[900px] w-full bg-white dark:bg-[#09090b] rounded-xl shadow-sm overflow-hidden border border-slate-200 dark:border-slate-800 transition-all">
      <div class="p-6 space-y-6">

        {#if !showSettings}
          <!-- Main UI -->
          <div class="flex items-center justify-end gap-1">
            <button on:click={cycleColorMode} class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-slate-100 dark:hover:bg-slate-800 h-8 w-8 text-slate-500 hover:text-slate-900 dark:hover:text-slate-50" title="外観モードの切り替え">
              {#if $colorMode === "light"}
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>
              {:else if $colorMode === "dark"}
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>
              {:else}
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/></svg>
              {/if}
            </button>
            <button on:click={openSettings} class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-slate-100 dark:hover:bg-slate-800 h-8 w-8 text-slate-500 hover:text-slate-900 dark:hover:text-slate-50" title="設定">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
            </button>
          </div>


          <div class="flex gap-6 h-[400px]">
            <!-- Left Panel: Action Buttons (Single Column) -->
            <div class="flex flex-col gap-2 w-[40%]">
              <button
                on:click={handleDelete}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-rose-50 dark:hover:bg-rose-950/20 hover:text-rose-600 dark:hover:text-rose-400 px-4 py-3 w-full"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
                <span class="truncate">一括削除</span>
                <span class="ml-auto text-[10px] font-normal text-slate-500 dark:text-slate-400">新しいフォルダー</span>
              </button>

              <button
                on:click={handleOrganize}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary-hover px-4 py-3 w-full"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-90 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/></svg>
                <span class="truncate">拡張子ごとに整理</span>
              </button>

              <button
                on:click={handleClassify}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 px-4 py-3 w-full"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
                <span class="truncate">内容で分類</span>
              </button>

              <button
                on:click={() => showTodayModal = true}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 px-4 py-3 w-full"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/><path d="M8 14h.01"/><path d="M12 14h.01"/><path d="M16 14h.01"/><path d="M8 18h.01"/><path d="M12 18h.01"/><path d="M16 18h.01"/></svg>
                <span class="truncate">今日のファイルを整理</span>
              </button>

              <div class="h-px bg-slate-100 dark:bg-slate-800 my-1"></div>

              <button
                on:click={handleEmptyRecycleBin}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 px-4 py-3 w-full"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
                <span class="truncate">ゴミ箱を空にする</span>
              </button>

              <button
                on:click={handleDeleteDuplicates}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-rose-50 dark:hover:bg-rose-950/20 hover:text-rose-600 dark:hover:text-rose-400 px-4 py-3 w-full"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.29 7 12 12 20.71 7"/><line x1="12" x2="12" y1="22" y2="12"/></svg>
                <span class="truncate">重複ファイルを削除</span>
              </button>

              <button
                on:click={handleUndo}
                class="inline-flex items-center justify-start gap-3 whitespace-nowrap rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] hover:bg-slate-100 dark:hover:bg-slate-800 px-4 py-3 w-full mt-auto"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-70 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/></svg>
                <span class="truncate">元に戻す</span>
              </button>
            </div>

            <!-- Right Panel: History -->
            <div class="flex-1 rounded-md border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-[#09090b] flex flex-col overflow-hidden">
              <div class="bg-slate-100 dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800 px-3 py-2 shrink-0 flex items-center justify-between">
                <span class="text-[10px] font-medium text-slate-500 uppercase tracking-wider">操作履歴</span>
                <span class="text-[9px] text-slate-400 font-mono">{history.length} 件</span>
              </div>
              <div class="flex-1 overflow-y-auto p-3 space-y-2 custom-scrollbar min-h-0">
                {#if history.length === 0}
                  <div class="h-full flex items-center justify-center">
                    <p class="text-[10px] text-slate-400 italic">履歴はありません</p>
                  </div>
                {/if}
                {#each history as item}
                  <div class="flex gap-2 text-[10px] animate-in fade-in duration-300">
                    <span class="text-slate-400 font-mono shrink-0">{item.time}</span>
                    <span class="text-slate-600 dark:text-slate-300">{item.msg}</span>
                  </div>
                {/each}
              </div>
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
            <button on:click={closeSettings} class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-slate-100 dark:hover:bg-slate-800 h-8 w-8 text-slate-500">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
            </button>
            <h2 class="text-xs font-semibold uppercase tracking-tight text-slate-900 dark:text-slate-50">設定</h2>
            <div class="w-8"></div>
          </div>

          <div class="space-y-6">
            <div class="space-y-4">
              <div class="flex items-center gap-4 border-b border-slate-100 dark:border-slate-800 pb-4">
                <label for="prefix" class="text-[10px] font-medium text-slate-500 uppercase tracking-wider w-1/4">整理接頭辞</label>
                <input
                  id="prefix"
                  type="text"
                  bind:value={$prefix}
                  class="flex h-8 w-[20%] rounded-md border border-slate-200 dark:border-slate-800 bg-transparent px-2 py-1 text-xs shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring text-center font-bold"
                />
                <p class="text-[9px] text-slate-400 flex-1 ml-4">拡張子ごとの整理時に付与されるプレフィックスです。</p>
              </div>

              <div class="flex items-center gap-4 border-b border-slate-100 dark:border-slate-800 pb-4">
                <label for="todayPrefix" class="text-[10px] font-medium text-slate-500 uppercase tracking-wider w-1/4">日次接頭辞</label>
                <input
                  id="todayPrefix"
                  type="text"
                  bind:value={$todayPrefix}
                  class="flex h-8 w-[20%] rounded-md border border-slate-200 dark:border-slate-800 bg-transparent px-2 py-1 text-xs shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring text-center font-bold"
                />
                <p class="text-[9px] text-slate-400 flex-1 ml-4">今日のファイル整理時に付与されるプレフィックスです。</p>
              </div>
            </div>

            <div class="space-y-4 pt-2">
              <div class="flex items-center gap-4">
                <label class="text-[10px] font-medium text-slate-500 uppercase tracking-wider w-1/4">外観モード</label>
                <select
                  class="flex h-8 w-[30%] items-center justify-between rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] px-3 py-2 text-xs shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                  bind:value={$colorMode}
                >
                  <option value="system">システム同期</option>
                  <option value="light">ライト</option>
                  <option value="dark">ダーク</option>
                </select>
              </div>

              <div class="flex items-center gap-4 border-b border-slate-100 dark:border-slate-800 pb-4">
                <label class="text-[10px] font-medium text-slate-500 uppercase tracking-wider w-1/4">テーマカラー</label>
                <div class="flex items-center gap-3 flex-1">
                  <select
                    class="flex h-8 w-[40%] items-center justify-between rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] px-3 py-2 text-xs shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                    value={$theme}
                    on:change={(e) => {
                      theme.set(e.target.value);
                      if (e.target.value !== 'theme-custom') {
                        customColor.set("");
                      }
                    }}
                  >
                    {#each presetColors as preset}
                      <option value={preset.class}>{preset.name}</option>
                    {/each}
                    <option value="theme-custom">Custom</option>
                  </select>
                  <input
                    type="color"
                    value={$customColor || presetColors.find(p => p.class === $theme)?.value || "#475569"}
                    on:input={(e) => {
                      customColor.set(e.target.value);
                      theme.set("theme-custom");
                    }}
                    class="h-8 w-12 cursor-pointer rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] p-1 shrink-0 shadow-sm"
                    title="カスタムカラー"
                  />
                </div>
              </div>
              <div class="rounded-md border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-[#09090b] p-3">
                <div class="max-h-[120px] overflow-y-auto space-y-1.5 custom-scrollbar min-h-[40px] mb-3">
                  {#if $customRules.length === 0}
                    <p class="text-[10px] text-slate-500 italic text-center py-2">ルールはありません</p>
                  {/if}
                  {#each $customRules as rule, i}
                    <div class="flex items-center gap-2 group">
                      <input type="text" class="w-1/3 text-[9px] font-bold truncate text-slate-700 dark:text-slate-300 bg-transparent border border-transparent hover:border-slate-300 dark:hover:border-slate-700 focus:border-slate-400 dark:focus:border-slate-600 focus:outline-none rounded px-1" value={rule.name} on:change={(e) => updateRuleName(i, e)} title={rule.name} />
                      <input type="text" class="flex-1 text-[9px] truncate text-slate-500 font-mono bg-transparent border border-transparent hover:border-slate-300 dark:hover:border-slate-700 focus:border-slate-400 dark:focus:border-slate-600 focus:outline-none rounded px-1" value={rule.extensions.join(', ')} on:change={(e) => updateRuleExts(i, e)} title={rule.extensions.join(', ')} />
                      <button on:click={() => removeCustomRule(i)} class="inline-flex items-center justify-center rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-rose-50 dark:hover:bg-rose-950/20 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 h-5 w-5">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                      </button>
                    </div>
                  {/each}
                </div>

                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={newRuleName}
                    placeholder="例: VideoProject"
                    class="flex h-6 w-1/3 rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] px-2 py-1 text-[9px] shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  />
                  <input
                    type="text"
                    bind:value={newRuleExts}
                    placeholder="拡張子 (例: mp4, mov)"
                    class="flex h-6 flex-1 rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] px-2 py-1 text-[9px] shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    on:keydown={(e) => e.key === 'Enter' && addCustomRule()}
                  />
                  <button on:click={addCustomRule} class="inline-flex items-center justify-center rounded-md text-[9px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary-hover h-6 px-3 whitespace-nowrap">追加</button>
                </div>
              </div>
            </div>

          </div>
        {/if}
      </div>
    </div>


    {#if contextMenu.show && !showSettings}
      <div
        class="fixed z-50 min-w-[200px] overflow-hidden rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] p-1 text-slate-950 dark:text-slate-50 shadow-md animate-in fade-in zoom-in-95 duration-100"
        style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
      >
        <button on:click={handleDelete} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2 text-rose-600 dark:text-rose-400">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
          一括削除
        </button>
        <button on:click={handleOrganize} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2 text-primary">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/></svg>
          拡張子ごとに整理
        </button>
        <button on:click={handleClassify} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
          内容で分類
        </button>
        <button on:click={() => showTodayModal = true} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/><path d="M8 14h.01"/><path d="M12 14h.01"/><path d="M16 14h.01"/><path d="M8 18h.01"/><path d="M12 18h.01"/><path d="M16 18h.01"/></svg>
          今日のファイルを整理
        </button>
        <div class="h-px bg-slate-200 dark:bg-slate-800 my-1"></div>
        <button on:click={handleEmptyRecycleBin} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
          ゴミ箱を空にする
        </button>
        <button on:click={handleDeleteDuplicates} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2 text-rose-600 dark:text-rose-400">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.29 7 12 12 20.71 7"/><line x1="12" x2="12" y1="22" y2="12"/></svg>
          重複ファイルを削除
        </button>
        <div class="h-px bg-slate-200 dark:bg-slate-800 my-1"></div>
        <button on:click={handleUndo} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/></svg>
          元に戻す
        </button>
        <div class="h-px bg-slate-200 dark:bg-slate-800 my-1"></div>
        <button on:click={openSettings} class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-slate-100 dark:hover:bg-slate-800 w-full text-left gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
          設定
        </button>
      </div>
    {/if}


    <!-- Today's File Organization Modal -->
    <!-- Operation Preview Modal -->
    {#if showPreviewModal}
      <div class="absolute inset-0 bg-slate-950/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <div class="bg-white dark:bg-[#09090b] w-full max-w-[320px] max-h-[80vh] rounded-lg border border-slate-200 dark:border-slate-800 shadow-2xl flex flex-col animate-in fade-in zoom-in duration-200">
          <div class="p-4 border-b border-slate-100 dark:border-slate-800 shrink-0">
            <h3 class="text-[11px] font-bold tracking-tight">{previewTitle} のプレビュー</h3>
            <p class="text-[9px] text-slate-500 mt-0.5">以下の移動が実行されます</p>
          </div>

          <div class="flex-1 overflow-y-auto p-4 space-y-1.5 custom-scrollbar">
            {#each previewItems as item}
              <div class="flex flex-col p-1.5 bg-slate-50 dark:bg-slate-900/50 rounded border border-slate-100 dark:border-slate-800/50">
                <div class="flex items-center gap-2 overflow-hidden">
                  <span class="text-[9px] font-mono truncate flex-1 text-slate-600 dark:text-slate-400">{item.from}</span>
                </div>
                <div class="flex items-center gap-2 mt-1">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-2.5 h-2.5 text-primary opacity-50 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>
                  <span class="text-[9px] font-bold truncate text-primary">{item.to}</span>
                </div>
              </div>
            {/each}
          </div>

          <div class="p-4 border-t border-slate-100 dark:border-slate-800 flex gap-2 shrink-0">
            <button
              on:click={() => showPreviewModal = false}
              class="flex-1 h-8 rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] text-[10px] font-medium hover:bg-slate-100 dark:hover:bg-slate-800"
            >
              キャンセル
            </button>
            <button
              on:click={() => { showPreviewModal = false; onPreviewConfirm(); }}
              class="flex-1 h-8 rounded-md bg-primary text-primary-foreground text-[10px] font-medium hover:bg-primary-hover shadow-sm"
            >
              実行する
            </button>
          </div>
        </div>
      </div>
    {/if}

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

  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .no-scrollbar {
    -ms-overflow-style: none;  /* IE and Edge */
    scrollbar-width: none;  /* Firefox */
  }

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

  .custom-scrollbar::-webkit-scrollbar {
    width: 2px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: theme('colors.slate.300');
    border-radius: 10px;
  }
  .custom-scrollbar:hover::-webkit-scrollbar-thumb {
    background: theme('colors.slate.400');
  }
  :global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb {
    background: theme('colors.slate.700');
  }
  :global(.dark) .custom-scrollbar:hover::-webkit-scrollbar-thumb {
    background: theme('colors.slate.600');
  }

  .animate-in {
    animation: animate-in 0.2s ease-out;
  }
  @keyframes animate-in {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
