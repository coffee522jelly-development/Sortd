with open('src/routes/+page.svelte', 'r') as f:
    content = f.read()

# Let's clean up the grid.
# Find:
#           <div class="space-y-6">
#             <!-- Desktop Organization Group -->
#             <div class="grid grid-cols-2 gap-4">
# ...
#             </div>
#
#             <div class="relative col-span-2">
# ...
#             </div>
#
#             <!-- Maintenance Group -->
#             <div class="grid grid-cols-2 gap-4">

# Replace with a single grid:

import re
old_structure = """          <div class="space-y-6">
            <!-- Desktop Organization Group -->
            <div class="grid grid-cols-2 gap-4">"""

new_structure = """          <div class="grid grid-cols-2 gap-4">
            <!-- Desktop Organization Group -->"""

content = content.replace(old_structure, new_structure)

old_mid = """            </div>

            <div class="relative col-span-2">
              <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-slate-100 dark:border-slate-800/50"></span></div>
              <div class="relative flex justify-center text-[9px] uppercase"><span class="bg-white dark:bg-[#09090b] px-2 text-slate-400">Other Utilities</span></div>
            </div>

            <!-- Maintenance Group -->
            <div class="grid grid-cols-2 gap-4">"""

new_mid = """            <div class="relative col-span-2 my-2">
              <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-slate-100 dark:border-slate-800/50"></span></div>
              <div class="relative flex justify-center text-[9px] uppercase"><span class="bg-white dark:bg-[#09090b] px-2 text-slate-400">Other Utilities</span></div>
            </div>

            <!-- Maintenance Group -->"""
content = content.replace(old_mid, new_mid)

with open('src/routes/+page.svelte', 'w') as f:
    f.write(content)
