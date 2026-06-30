+++
title = ""
date = 2026-06-24T20:23:58+00:00
description = "And another bash alias: Better word movement: treat aaabbbccc as ONE word Ctrl + Left → move left by \"word\" (including underscores) Ctrl + Right → move right by \"word\" (including underscores) if [[…"

[taxonomies]
days = ["2026-06-24"]
tags = ["bash", "alias"]

[extra]
id = 1864
day = "2026-06-24"
tg_url = "https://t.me/vitaly_zdanevich_chan/1864"
next_id = 1865
next_title = ""
prev_id = 1863
prev_title = ""
views = 11
ids = [1864]
+++

And another {{ tag(t="bash") }} {{ tag(t="alias") }}:

```
# Better word movement: treat aaa_bbb_ccc as ONE word
# Ctrl + Left  → move left by "word" (including underscores)
# Ctrl + Right → move right by "word" (including underscores)
if [[ $- == *i* ]]; then
  bind '"\e[1;5D": shell-backward-word'   # Ctrl + Left Arrow
  bind '"\e[1;5C": shell-forward-word'    # Ctrl + Right Arrow
fi
```
