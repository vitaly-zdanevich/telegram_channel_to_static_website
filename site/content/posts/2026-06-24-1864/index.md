+++
title = "And another bash alias"
date = 2026-06-24T20:23:58+00:00
description = "And another bash alias: Better word movement: treat aaabbbccc as ONE word Ctrl + Left → move left by \"word\" (including underscores) Ctrl + Right → move right by \"word\" (including underscores) if [[…"

[taxonomies]
tags = ["bash", "alias"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1864"
next_id = 1865
next_title = "Wow, Gemini generates good logos, tried it for the first time"
prev_id = 1863
prev_title = "My new bash alias"
views = 8
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
