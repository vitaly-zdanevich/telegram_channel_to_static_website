+++
title = ""
date = 2026-03-05T07:10:14+00:00
description = "bash I love cli, scripts, and sometimes I want my script to accept an argument that is the same as the folder name. How to pass that current folder name to the script? upload.py file.pdf --category…"

[taxonomies]
days = ["2026-03-05"]
tags = ["bash", "cli"]

[extra]
id = 1334
day = "2026-03-05"
tg_url = "https://t.me/vitaly_zdanevich_chan/1334"
next_id = 1335
next_title = ""
next_body = "I use #bash history Ctrl-R a lot, also with #fzf and other helpers, and have bash aliases, that are just one letter, and I do not want to pollute my bash #history with it, so I found the solution - the bash function/alias that delete itself from the history, for example:\ns() {\ngit status\nhistory -d \"$(history 1 | awk '{print $1}')\"\n# delete from history\n}"
prev_id = 1333
prev_title = ""
prev_body = "#design\n#graph\n#wikimedia"
views = 9
ids = [1334]

[[extra.related]]
path = "@/posts/2026-03-05-1336/index.md"
label = "...one of my #bash aliases: to count files here: c { ls -1  wc -…"

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
path = "@/posts/2025-02-12-357/index.md"
label = "#gui #cli"
+++

{{ tag(t="bash") }}  

I love {{ tag(t="cli") }}, scripts, and sometimes I want my script to accept an argument that is the same as the folder name. How to pass that current folder name to the script?  

```
upload.py file.pdf --category "${PWD##*/}"
```

Yep, it works.
