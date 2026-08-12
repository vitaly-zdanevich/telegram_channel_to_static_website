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
next_body = "Wow, Gemini generates good logos, tried it for the first time\n#gemini\n#logo\n#telegrambot\n#wikimediacommons"
prev_id = 1863
prev_title = ""
prev_body = "My new #bash #alias\nbind '\"ej\": \"!#:$ e^\"'\n# Alt J repeats prev word\n# This is useful when, for example, you want to rename a file,\n# for example - to adjust it file extension.\n/.inputrc syntax is simpler:\n\"ej\": \"!#:$ e^\""
views = 26
ids = [1864]

[[extra.related]]
path = "@/posts/2026-06-24-1863/index.md"
label = "My new #bash #alias bind '\"ej\": \"!#:$ e^\"' # Alt J repeats prev…"

[[extra.related]]
path = "@/posts/2026-04-25-1683/index.md"
label = "...and another useful #bash #alias to #remove the text inside si…"

[[extra.related]]
path = "@/posts/2026-04-25-1682/index.md"
label = "My new great #bash #alias actually a #hotkey for faster #cd # Li…"

[[extra.related]]
path = "@/posts/2026-03-05-1336/index.md"
label = "...one of my #bash aliases: to count files here: c { ls -1  wc -…"

[[extra.related]]
path = "@/posts/2026-03-05-1334/index.md"
label = "#bash I love #cli, scripts, and sometimes I want my script to ac…"
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
