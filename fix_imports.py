with open('src/routes/+page.svelte', 'r') as f:
    content = f.read()

import re
content = re.sub(
    r"import \{ getCurrentWindow \} from '@tauri-apps/api/window';\nimport \{ LogicalSize \} from '@tauri-apps/api/dpi';\n",
    r"",
    content
)

with open('src/routes/+page.svelte', 'w') as f:
    f.write(content)
