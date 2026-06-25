import re

with open('src/routes/+page.svelte', 'r') as f:
    content = f.read()

# Add a state for custom color
content = re.sub(
    r'  let theme = persisted\("sortd_theme", "theme-slate"\);',
    r'  let theme = persisted("sortd_theme", "theme-slate");\n  let customColor = persisted("sortd_custom_color", "");',
    content
)

# Update the main wrapper to support inline styles for custom colors
content = re.sub(
    r'<div class=\{\$theme\}>',
    r'<div class={$theme} style={$customColor ? `--theme-primary: ${$customColor}; --theme-primary-hover: ${$customColor}dd;` : ""}>',
    content
)

# Modify the themes to map to actual hex values so we can use them in the color picker,
# or just provide a list of hex values for the presets.
themes_decl = """  const presetColors = [
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
  ];"""

content = re.sub(
    r'  const themes = \[\n    "theme-slate",.*?\n  \];',
    themes_decl,
    content,
    flags=re.DOTALL
)

# Replace the color UI
old_color_ui = """              <div class="grid grid-cols-8 gap-2">
                {#each themes as t}
                  <button
                    on:click={() => theme.set(t)}
                    class="w-4 h-4 rounded-full border border-slate-200 dark:border-slate-800 transition-all {t} bg-primary { $theme === t ? 'ring-2 ring-ring ring-offset-2 ring-offset-white dark:ring-offset-[#09090b] scale-110' : 'hover:scale-110' }"
                  ></button>
                {/each}
              </div>"""

new_color_ui = """              <div class="flex items-center gap-3">
                <input
                  type="color"
                  value={$customColor || presetColors.find(p => p.class === $theme)?.value || "#475569"}
                  on:input={(e) => {
                    customColor.set(e.target.value);
                    theme.set("theme-custom");
                  }}
                  class="w-8 h-8 rounded cursor-pointer border-0 p-0 bg-transparent"
                  title="カスタムカラーを選択"
                />
                <select
                  class="flex h-8 w-full items-center justify-between rounded-md border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#09090b] px-3 py-2 text-xs shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                  value={$theme}
                  on:change={(e) => {
                    const val = e.target.value;
                    if (val !== "theme-custom") {
                      customColor.set("");
                      theme.set(val);
                    }
                  }}
                >
                  <option value="theme-custom" disabled hidden={$theme !== "theme-custom"}>カスタムカラー</option>
                  {#each presetColors as p}
                    <option value={p.class}>{p.name}</option>
                  {/each}
                </select>
              </div>"""

content = content.replace(old_color_ui, new_color_ui)

with open('src/routes/+page.svelte', 'w') as f:
    f.write(content)
