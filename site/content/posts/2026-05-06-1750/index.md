+++
title = ""
date = 2026-05-06T20:05:38+00:00
description = "This is how in go we remove an element from a collection func rm(i int, lists[]ListNode) []ListNode { return append(lists[:i], lists[i+1:]...) } Usually its a library call."

[taxonomies]
days = ["2026-05-06"]
tags = ["go"]

[extra]
id = 1750
day = "2026-05-06"
tg_url = "https://t.me/vitaly_zdanevich_chan/1750"
next_id = 1751
next_title = ""
next_body = "#7мая2026 (чт) 21.30-01:00 айтишная посиделка в Still Young Bar\n#безоплаты\nДоклады:\n1. [Soft] «Создание телеграмм-бота для управления заметками в программе Evernote!. Вайб кодинг.» (💻@vitalyzdanevich)\n🤔 Ты тоже можешь выступить с докладом в неформальной обстановке на большом экране.\nДокладчику – пивас в подарок! ☕️🍺\n➡️Расписание\n🗓 21:00-21:30 - Сбор\n👨‍🏫22:30-22:30 - Доклад\n🤼22:30-до последнего итишника - Разговоры о высоком/Караоке\n📍Адрес: Still Young Bar (ул. Генерала Мазниашвили 48)\n⏰ 21:00-до последнего итишника\n💬 Все вопросы – в личку: @marstut или @AMVavilov"
prev_id = 1749
prev_title = ""
prev_body = "#leetcode\n#validation"
views = 27
ids = [1750]
+++

This is how in {{ tag(t="go") }} we remove an element from a collection  

```
func rm(i int, lists[]*ListNode) []*ListNode {
    return append(lists[:i], lists[i+1:]...)
}
```

Usually its a library call.
