+++
title = ""
date = 2026-06-24T20:19:32+00:00
description = "My new bash alias bind '\"ej\": \"!:$ e^\"' Alt J repeats prev word This is useful when, for example, you want to rename a file, for example - to adjust it file extension. /.inputrc syntax is simpler:…"

[taxonomies]
days = ["2026-06-24"]
tags = ["bash", "alias"]

[extra]
id = 1863
day = "2026-06-24"
tg_url = "https://t.me/vitaly_zdanevich_chan/1863"
next_id = 1864
next_title = ""
next_body = "And another #bash #alias:\n# Better word movement: treat aaabbbccc as ONE word\n# Ctrl + Left → move left by \"word\" (including underscores)\n# Ctrl + Right → move right by \"word\" (including underscores)\nif [[ $- == i ]]; then\nbind '\"e[1;5D\": shell-backward-word' # Ctrl + Left Arrow\nbind '\"e[1;5C\": shell-forward-word' # Ctrl + Right Arrow\nfi"
prev_id = 1861
prev_title = ""
prev_body = "#26июня2026 (пт) 21.30-23:00 айтишная посиделка в Friends club\n#безоплаты\nДоклады:\n1. [Soft] «Telegram bot для RuTracker - который не только ищет, как прочие, но и скачивает» (💻 @vitalyzdanevich)\n2. [Soft] batbus.app - автобусы в Батуми. Итоги 2-х месяцев. (🧖🏼@marstut) PS: Если успеем\n➡️Расписание\n🗓 21:00-21:30 - Сбор\n💬 21:30-22:00 - Знакомимся с Крякой\n🍺22:00-22:10 - Запасаемся пивом/медовухой/кальяном\n👨‍🏫22:10 - 22:55 Конкурс мокрых маек Доклад\n🍺23:00-до последнего итишника - Разговоры о высоком/Игры в шахматы/Нетворкинг\n📍Адрес: Friends club (Мемеда Абашидзе, 57)\n⏰ 21:30-23:00\n💬 Все вопросы – в личку: @marstut"
views = 23
ids = [1863]

[[extra.related]]
path = "@/posts/2026-06-24-1864/index.md"
label = "And another #bash #alias: # Better word movement: treat aaabbbcc…"
date = "2026-06-24"

[[extra.related]]
path = "@/posts/2026-04-25-1683/index.md"
label = "...and another useful #bash #alias to #remove the text inside si…"
date = "2026-04-25"

[[extra.related]]
path = "@/posts/2026-04-25-1682/index.md"
label = "My new great #bash #alias actually a #hotkey for faster #cd # Li…"
date = "2026-04-25"

[[extra.related]]
path = "@/posts/2026-03-05-1336/index.md"
label = "...one of my #bash aliases: to count files here: c { ls -1  wc -…"
date = "2026-03-05"

[[extra.related]]
path = "@/posts/2026-03-05-1334/index.md"
label = "#bash I love #cli, scripts, and sometimes I want my script to ac…"
date = "2026-03-05"
+++

My new {{ tag(t="bash") }} {{ tag(t="alias") }}  

```
bind '"\ej": "!#:$ \e^"'
# Alt J repeats prev word
# This is useful when, for example, you want to rename a file,
# for example - to adjust it file extension.
```

`~/.inputrc` syntax is simpler:  

```
"\ej": "!#:$ \e^"
```
