+++
title = ""
date = 2026-04-25T03:33:17+00:00
description = "My new great bash alias (actually a hotkey) for faster cd Like cd, but accept a path with spaces - so no quotation is needed How to use it: paste/type a path and press Ctrl-g bind \"\"C-g\": \"C-acd…"

[taxonomies]
days = ["2026-04-25"]
tags = ["bash", "alias", "hotkey", "cd"]

[extra]
id = 1682
day = "2026-04-25"
tg_url = "https://t.me/vitaly_zdanevich_chan/1682"
next_id = 1683
next_title = ""
next_body = "...and another useful #bash #alias to #remove the text inside single #quotes by esc-k:\n# Clear the content between single quotes while preserving the quotes\nclearinsidequotes() {\nlocal line=\"${READLINELINE}\"\n# Only run if the line contains at least two single quotes\nif [[ \"${line}\" == \"'\"\"'\" ]]; then\n# prefix: everything before the first quote\nlocal prefix=\"${line%%'}\"\n# suffix: everything after the last quote\nlocal suffix=\"${line##'}\"\n# Reconstruct the line with empty quotes\nREADLINELINE=\"${prefix}''${suffix}\"\n# Move the cursor back inside the empty quotes\nREADLINEPOINT=$((${#prefix} + 1))\nfi\n}\n# Bind the function to Alt-k (Escape + k)\nbind -x '\"ek\": clearinsidequotes'"
prev_id = 1681
prev_title = ""
prev_body = "#serp\n#armiesofexigo\n#google found a quote that exists on #youtube only"
views = 18
ids = [1682]

[[extra.related]]
path = "@/posts/2026-06-24-1864/index.md"
label = "And another #bash #alias: # Better word movement: treat aaabbbcc…"

[[extra.related]]
path = "@/posts/2026-06-24-1863/index.md"
label = "My new #bash #alias bind '\"ej\": \"!#:$ e^\"' # Alt J repeats prev…"

[[extra.related]]
path = "@/posts/2026-02-10-1103/index.md"
label = "My new #bash alias: one function to go to the next folder like f…"

[[extra.related]]
path = "@/posts/2026-04-25-1683/index.md"
label = "...and another useful #bash #alias to #remove the text inside si…"

[[extra.related]]
path = "@/posts/2026-03-05-1336/index.md"
label = "...one of my #bash aliases: to count files here: c { ls -1  wc -…"
+++

My new great {{ tag(t="bash") }} {{ tag(t="alias") }} (actually a {{ tag(t="hotkey") }}) for faster {{ tag(t="cd") }}  

```
# Like cd, but accept a path with spaces - so no quotation is needed
# How to use it: paste/type a path and press Ctrl-g
bind "\"\C-g\": \"\C-acd '\C-e'\C-j\""
# How it works: it executes multiple commands:
# Ctrl-a to go to the line start
# Types cd '
# Ctrl-e to go to the line end
# Add single quote
# Ctrl-j hits the Enter
# =====
# I tried an alias/function
# `cd "$*"`
# and
# `cd "$1"`
# but got error when path contains `(`
```
