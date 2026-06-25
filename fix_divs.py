with open('src/routes/+page.svelte', 'r') as f:
    content = f.read()

# I need to remove one closing div because we merged two grid divs into one.
import re

content = re.sub(r'              </button>\n            </div>\n          </div>', r'              </button>\n            </div>', content)

with open('src/routes/+page.svelte', 'w') as f:
    f.write(content)
