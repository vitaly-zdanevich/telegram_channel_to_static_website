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
views = 19
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
