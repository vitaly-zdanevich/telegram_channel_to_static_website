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
prev_id = 1333
prev_title = ""
views = 9
ids = [1334]
+++

{{ tag(t="bash") }}

I love {{ tag(t="cli") }}, scripts, and sometimes I want my script to accept an argument that is the same as the folder name. How to pass that current folder name to the script?

```
upload.py file.pdf --category "${PWD##*/}"
```

Yep, it works.
