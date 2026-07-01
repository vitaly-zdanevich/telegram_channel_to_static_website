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
prev_id = 1335
prev_title = ""
views = 9
ids = [1336]
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
