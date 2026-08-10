+++
title = ""
date = 2026-03-05T07:13:57+00:00
description = "I use bash history Ctrl-R a lot, also with fzf and other helpers, and have bash aliases, that are just one letter, and I do not want to pollute my bash history with it, so I found the solution - the…"

[taxonomies]
days = ["2026-03-05"]
tags = ["bash", "fzf", "history"]

[extra]
id = 1335
day = "2026-03-05"
tg_url = "https://t.me/vitaly_zdanevich_chan/1335"
next_id = 1336
next_title = ""
next_body = "...one of my #bash aliases: to count files here:\nc() {\nls -1 | wc -l\n# count files here\nhistory -d \"$(history 1 | awk '{print $1}')\"\n# delete from history\n}"
prev_id = 1334
prev_title = ""
prev_body = "#bash\nI love #cli, scripts, and sometimes I want my script to accept an argument that is the same as the folder name. How to pass that current folder name to the script?\nupload.py file.pdf --category \"${PWD##/}\"\nYep, it works."
views = 9
ids = [1335]

[[extra.related]]
path = "@/posts/2026-02-20-1119/index.md"
label = "#bash #history #mcfly: ctrl-r replacement with \"suggestions are…"

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
path = "@/posts/2026-03-05-1334/index.md"
label = "#bash I love #cli, scripts, and sometimes I want my script to ac…"
+++

I use {{ tag(t="bash") }} history Ctrl-R a lot, also with {{ tag(t="fzf") }} and other helpers, and have bash aliases, that are just one letter, and I do not want to pollute my bash {{ tag(t="history") }} with it, so I found the solution - the bash function/alias that delete itself from the history, for example:  

```
s() {
       git status

       history -d "$(history 1 | awk '{print $1}')"
       # delete from history
}
```
