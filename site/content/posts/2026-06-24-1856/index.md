+++
title = ""
date = 2026-06-24T15:14:35+00:00
description = "armiesofexigo tool Софт Там к софту есть инструкции но вот еще своими словами написал в файле list.LST все пути к игровым файлам === чтобы извлечь все файлы в консоли: \"tools/ork/orkdumper/orkdec\"…"

[taxonomies]
days = ["2026-06-24"]
tags = ["armies_of_exigo", "tool"]

[extra]
id = 1856
day = "2026-06-24"
tg_url = "https://t.me/vitaly_zdanevich_chan/1856"
next_id = 1860
next_title = ""
prev_id = 1855
prev_title = ""
views = 6
ids = [1856, 1857, 1858, 1859]
+++

{{ tag(t="armies_of_exigo") }}
{{ tag(t="tool") }}

Софт

Там к софту есть инструкции но вот еще своими словами написал

в файле list.LST все пути к игровым файлам

===

чтобы извлечь все файлы в консоли:
"tools/ork/ork\_dumper/orkdec" Data.ork list.LST

===

чтобы заменить файл в игре на примере диалога выбора персонажа:
1. создаем любой файл .LST пусть test.LST в папке с игрой
2. в нем пишем список файлов которые хотим заменить (каждый с новой строки). Например:
sound\\beasts\\bwkr\_select01.ogg
4. в папке с игрой по этому пути (sound\\beasts\\bwkr\_select01.ogg) вставляем новый файл на замену
5. в консоли:
"tools/ork/ork\_compiler/orkcmp" -g DataX.ork test.LST
6. в папке с игрой появится файл DataX.ork переименовываем его в Data3.ork
7. все

📎 Armies of Exigo.7z *(not archived)*
