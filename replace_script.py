import re

with open('src/routes/+page.svelte', 'r') as f:
    content = f.read()

# Remove the window resizing logic in openSettings and closeSettings
content = re.sub(
    r'async function openSettings\(\) \{\n\s*showSettings = true;\n\s*try \{\n\s*await getCurrentWindow\(\)\.setSize\(new LogicalSize\(960, 580\)\);\n\s*\} catch \(e\) \{\n\s*console\.error\(e\);\n\s*\}\n\s*\}',
    r'async function openSettings() {\n    showSettings = true;\n  }',
    content
)

content = re.sub(
    r'async function closeSettings\(\) \{\n\s*showSettings = false;\n\s*try \{\n\s*await getCurrentWindow\(\)\.setSize\(new LogicalSize\(360, 580\)\);\n\s*\} catch \(e\) \{\n\s*console\.error\(e\);\n\s*\}\n\s*\}',
    r'async function closeSettings() {\n    showSettings = false;\n  }',
    content
)

# Change the max-width logic on the main container
content = content.replace(
    '''<div class="{showSettings ? 'max-w-[880px]' : 'max-w-[280px]'} w-full bg-white dark:bg-[#09090b] rounded-xl shadow-sm overflow-hidden border border-slate-200 dark:border-slate-800 transition-all">''',
    '''<div class="max-w-[880px] w-full bg-white dark:bg-[#09090b] rounded-xl shadow-sm overflow-hidden border border-slate-200 dark:border-slate-800 transition-all">'''
)

# Update the main UI layout to be a grid
old_main_ui = """          <div class="space-y-4">
            <!-- Desktop Organization Group -->
            <div class="grid gap-3">"""
new_main_ui = """          <div class="space-y-6">
            <!-- Desktop Organization Group -->
            <div class="grid grid-cols-2 gap-4">"""
content = content.replace(old_main_ui, new_main_ui)

old_divider = """            <div class="relative">
              <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-slate-100 dark:border-slate-800/50"></span></div>
              <div class="relative flex justify-center text-[9px] uppercase"><span class="bg-white dark:bg-[#09090b] px-2 text-slate-400">Other Utilities</span></div>
            </div>"""
new_divider = """            <div class="relative col-span-2">
              <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-slate-100 dark:border-slate-800/50"></span></div>
              <div class="relative flex justify-center text-[9px] uppercase"><span class="bg-white dark:bg-[#09090b] px-2 text-slate-400">Other Utilities</span></div>
            </div>"""
content = content.replace(old_divider, new_divider)

old_maint_group = """            <!-- Maintenance Group -->
            <div class="grid gap-3">"""
new_maint_group = """            <!-- Maintenance Group -->
            <div class="grid grid-cols-2 gap-4">"""
content = content.replace(old_maint_group, new_maint_group)

old_undo = """              <button
                on:click={handleUndo}
                class="inline-flex items-center justify-center gap-2 mt-2 whitespace-nowrap text-[10px] font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 text-slate-400 dark:text-slate-600 hover:text-primary uppercase tracking-wider underline underline-offset-4 decoration-slate-200 dark:decoration-slate-800"
              >"""
new_undo = """              <button
                on:click={handleUndo}
                class="col-span-2 inline-flex items-center justify-center gap-2 mt-2 whitespace-nowrap text-[10px] font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 text-slate-400 dark:text-slate-600 hover:text-primary uppercase tracking-wider underline underline-offset-4 decoration-slate-200 dark:decoration-slate-800"
              >"""
content = content.replace(old_undo, new_undo)

# Remove the inner space-y-4 from the main ui wrapper since we want to move the divider into the grid itself
# Actually, the grid is in two divs. It's better to combine them into one grid for true grid layout.
with open('src/routes/+page.svelte', 'w') as f:
    f.write(content)
