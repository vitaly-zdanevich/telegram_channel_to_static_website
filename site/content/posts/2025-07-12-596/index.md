+++
title = ""
date = 2025-07-12T08:23:34+00:00
description = "My another userscript: small toggle for darkmode on stackexchange // ==UserScript== // @name StackExchange dark mode work-in-progress // @version 2025july4 // @description From // @author daniel.z.tg…"

[taxonomies]
days = ["2025-07-12"]
tags = ["userscript", "dark_mode", "stackexchange"]

[extra]
id = 596
day = "2025-07-12"
tg_url = "https://t.me/vitaly_zdanevich_chan/596"
next_id = 597
next_title = ""
next_body = "#music\n#anime\n#susumuhirasawa"
prev_id = 595
prev_title = ""
prev_body = "#preservation\n#belarus\n#library\n#science\nДля меня было открытием, что абсолютное большинство научных материалов до сих пор не в сети. Я работал в Центральной научной библиотеке Академии наук и понял, что до 95% всего научного контента не попадает в сеть. Все эти статьи, документация — они просто исчезают без следов. И хорошо, если это пишут для 200, а часто только для 50 человек. И не только в Беларуси, а во всём мире.\nЕсть ещё то, что называют «смертью интернета»: проходит десять лет — и почти ничего из того времени не находишь [в интернет-источниках]. Многие из источников, которыми я пользовался еще 3 — 5 лет назад, уже недоступны. Мы все смеёмся над газетами, журналами, бумажными носителями, но они на самом деле держатся дольше, чем интернет — 10-20 лет.\nВикипедия — это инструмент, который позволяет хранить материалы в удобном формате, иначе они просто исчезли бы."
views = 45
ids = [596]
+++

My another {{ tag(t="userscript") }}: small toggle for {{ tag(t="dark_mode") }} on {{ tag(t="stackexchange") }}  

<https://gitlab.com/vitaly-zdanevich-userscripts/stackexchange>  

```
// ==UserScript==
// @name         StackExchange dark mode work-in-progress
// @version      2025july4
// @description  From https://meta.stackexchange.com/a/407981/290021
// @author       daniel.z.tg and Vitaly Zdanevich
// @match        https://*.askubuntu.com/*
// @match        https://*.mathoverflow.net/*
// @match        https://*.serverfault.com/*
// @match        https://*.stackapps.com/*
// @match        https://*.stackexchange.com/*
// @match        https://superuser.com/*
// @run-at       document-body
// ==/UserScript==

// NOT working for all sites
document.body.classList.add('theme-dark');
```

<https://gitlab.com/vitaly-zdanevich-userscripts/stackexchange>
