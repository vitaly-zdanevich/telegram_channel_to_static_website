+++
title = ""
date = 2026-01-22T20:44:49+00:00
description = "I am on gentoo because it compiles for my CPU (-march=native and other flags - not generic x64, but I did not measure the performance numbers), and another point - USE flags and the ability to apply…"

[taxonomies]
days = ["2026-01-22"]
tags = ["gentoo"]

[extra]
id = 931
day = "2026-01-22"
tg_url = "https://t.me/vitaly_zdanevich_chan/931"
og_image = "01.jpg"
next_id = 933
next_title = ""
next_body = "#23января2026 (пт) 21.30-01:00 Айтишные посиделки в Laboratory Bar\n#безоплаты\nДоклады:\n[Soft] \"Про Codex от OpenAI: кейс использования: я пишу тесты - он сложные regular expressions, для автоматического редактирования Википедии - когда сайт переехал на другой домен\", Виталий Зданевич\n🤔 Ты тоже можешь выступить с докладом в неформальной обстановке на большом экране.\nДокладчику – пивас в подарок! ☕️🍺➡️Пивко на кране (Lager/IPA), мягкие диванчики и обновленный интеръер=)\nУже целый год мы проводим Friday-IT сходки в Laboratory Bar! За это время было рассказано и показано более 150 уникальных докладов 🔥 на самые разные темы.\n➡️Расписание\n🗓 21:00 - Сбор\n💬 21:30 - Знакомимся с Крякой\n🍺22:00 - Запасаемся пивом/медовухой/кальяном\n👨‍🏫22:10 - Конкурс мокрых маек Первый доклад\n🍺23:10 - Возобновляем запасы пива/кальяна\n👨‍🏫23:15 - Лучший в городе нетворкинг Айтишников\n🤼00:00 - Разговоры о высоком/Игры в шахматы\n📍Адрес: Laboratory bar (Генерала Мазниашвили 66)\n⏰ 21:30-01:00\n💬 Все вопросы – в личку:…"
prev_id = 930
prev_title = ""
prev_body = "Did you know about git notes?\nAdds, removes, or reads notes attached to objects, without touching the objects themselves.\nBy default, notes are saved to and read from refs/notes/commits, but this default can be overridden. See the OPTIONS, CONFIGURATION, and ENVIRONMENT sections below. If this ref does not exist, it will be quietly created when it is first needed to store a note.\nA typical use of notes is to supplement a commit message without changing the commit itself. Notes can be shown by git log along with the original commit message. To distinguish these notes from the message stored in the commit object, the notes are indented like the message, after an unindented line saying \"Notes (&lt;refname&gt;):\" (or \"Notes:\" for refs/notes/commits).\nNotes can also be added to patches prepared with git format-patch by using the --notes option. Such notes are added as a patch commentary after a three dash separator line."
views = 13
ids = [931]
+++

I am on {{ tag(t="gentoo") }} because it compiles for my CPU (`-march=native` and other flags - not generic x64, but I did not measure the performance numbers), and another point - [USE flags](https://wiki.gentoo.org/wiki/Handbook:AMD64/Working/USE) and the ability to apply source based patches. For example in Firefox in `about:firefoxview` in `Open tabs` we have only 7 elements and no preferences to increase that number, so I found it in the source code and produced a simple patch for my system - and on every update the Portage package manager will try to apply that patch.

![](01.jpg)

![](02.jpg)
