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
prev_id = 595
prev_title = ""
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
