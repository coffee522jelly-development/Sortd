with open('src/routes/+page.svelte', 'r') as f:
    content = f.read()

# The undo button should not be col-span-2 if we want it centered nicely below the other buttons.
# Actually, since it's inside a grid, col-span-2 is fine, but it needs to be centered.
# Or we can pull it out of the grid. Let's pull it out of the grid.

import re
content = re.sub(
    r'(              <button\n                on:click=\{handleEmptyRecycleBin\}[\s\S]*?              </button>)\n\n              <button\n                on:click=\{handleUndo\}\n                class="col-span-2 inline-flex items-center justify-center gap-2 mt-2 whitespace-nowrap text-\[10px\] font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 text-slate-400 dark:text-slate-600 hover:text-primary uppercase tracking-wider underline underline-offset-4 decoration-slate-200 dark:decoration-slate-800"\n              >\n                <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10h10a8 8 0 0 1 8 8v2"/><path d="m3 10 6 6"/><path d="m3 10 6-6"/></svg>\n                元に戻す\n              </button>\n            </div>',
    r'\1\n            </div>\n\n            <div class="flex justify-center">\n              <button\n                on:click={handleUndo}\n                class="inline-flex items-center justify-center gap-2 mt-2 whitespace-nowrap text-[10px] font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 text-slate-400 dark:text-slate-600 hover:text-primary uppercase tracking-wider underline underline-offset-4 decoration-slate-200 dark:decoration-slate-800"\n              >\n                <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10h10a8 8 0 0 1 8 8v2"/><path d="m3 10 6 6"/><path d="m3 10 6-6"/></svg>\n                元に戻す\n              </button>\n            </div>',
    content
)

with open('src/routes/+page.svelte', 'w') as f:
    f.write(content)
