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
prev_id = 1334
prev_title = ""
views = 9
ids = [1335]
+++

I use {{ tag(t="bash") }} history Ctrl-R a lot, also with {{ tag(t="fzf") }} and other helpers, and have bash aliases, that are just one letter, and I do not want to pollute my bash {{ tag(t="history") }} with it, so I found the solution - the bash function/alias that delete itself from the history, for example:

```
s() {
       git status

       history -d "$(history 1 | awk '{print $1}')"
       # delete from history
}
```
