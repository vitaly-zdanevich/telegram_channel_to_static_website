+++
title = ""
date = 2026-03-05T07:18:20+00:00
description = "...one of my bash aliases: to count files here: c() { ls -1 | wc -l count files here history -d \"$(history 1 | awk '{print $1}')\" delete from history }"

[taxonomies]
days = ["2026-03-05"]
tags = ["bash"]

[extra]
id = 1336
day = "2026-03-05"
tg_url = "https://t.me/vitaly_zdanevich_chan/1336"
next_id = 1337
next_title = ""
next_body = "#nest\n#cementery\n#belarus\n#globustut\n#year2005\nSource"
prev_id = 1335
prev_title = ""
prev_body = "I use #bash history Ctrl-R a lot, also with #fzf and other helpers, and have bash aliases, that are just one letter, and I do not want to pollute my bash #history with it, so I found the solution - the bash function/alias that delete itself from the history, for example:\ns() {\ngit status\nhistory -d \"$(history 1 | awk '{print $1}')\"\n# delete from history\n}"
views = 9
ids = [1336]

[[extra.related]]
path = "@/posts/2026-06-24-1864/index.md"
label = "And another #bash #alias: # Better word movement: treat aaabbbcc…"
date = "2026-06-24"

[[extra.related]]
path = "@/posts/2026-06-24-1863/index.md"
label = "My new #bash #alias bind '\"ej\": \"!#:$ e^\"' # Alt J repeats prev…"
date = "2026-06-24"

[[extra.related]]
path = "@/posts/2026-03-05-1334/index.md"
label = "#bash I love #cli, scripts, and sometimes I want my script to ac…"
date = "2026-03-05"

[[extra.related]]
path = "@/posts/2026-02-10-1103/index.md"
label = "My new #bash alias: one function to go to the next folder like f…"
date = "2026-02-10"

[[extra.related]]
path = "@/posts/2026-03-05-1335/index.md"
label = "I use #bash history Ctrl-R a lot, also with #fzf and other helpe…"
date = "2026-03-05"
+++

...one of my {{ tag(t="bash") }} aliases: to count files here:  

```
c() {
       ls -1 | wc -l
       # count files here

       history -d "$(history 1 | awk '{print $1}')"
       # delete from history
}
```
