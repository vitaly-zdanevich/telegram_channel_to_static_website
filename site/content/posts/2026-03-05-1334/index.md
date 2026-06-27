+++
title = "I love cli, scripts, and sometimes I want my script to accept an argument that is the same as the folder name."
date = 2026-03-05T07:10:14+00:00
description = "bash I love cli, scripts, and sometimes I want my script to accept an argument that is the same as the folder name. How to pass that current folder name to the script? upload.py file.pdf --category…"

[taxonomies]
tags = ["bash", "cli"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1334"
next_id = 1335
next_title = "I use bash history Ctrl-R a lot, also with fzf and other helpers, and have bash aliases, that are just one letter, and I do not want to pollute my bash history with it, so I found the solution - the…"
prev_id = 1333
prev_title = "design graph wikimedia"
views = 9
ids = [1334]
+++

{{ tag(t="bash") }}

I love {{ tag(t="cli") }}, scripts, and sometimes I want my script to accept an argument that is the same as the folder name. How to pass that current folder name to the script?

```
upload.py file.pdf --category "${PWD##*/}"
```

Yep, it works.
