+++
title = ""
date = 2026-06-05T18:31:37+00:00
description = "My yet another project: @wikipediaunofficialbot Built with llm gpt 5.5 xhigh"

[taxonomies]
days = ["2026-06-05"]
tags = ["llm", "gpt"]

[extra]
id = 1805
day = "2026-06-05"
tg_url = "https://t.me/vitaly_zdanevich_chan/1805"
next_id = 1806
next_title = ""
next_body = "Wow, about #telegram bots: you can bypass 50 MB response limit - by hosting their bot software. AWS Lambda is not possible here - it wants persistent polling.\nAnd it simplify the architecture - less nodes in the network chain - when Telegram server and your bot software on the same machine.\n#gemini about why not #lambda:\nTo understand why, you have to look at how Telegram's protocols work beneath the surface.\nWhen you use the standard public API (api.telegram.org), your bot makes simple, stateless HTTP requests. However, Telegram's core servers do not speak HTTP; they communicate using a custom, highly encrypted, and stateful protocol called MTProto.\nThe public api.telegram.org server acts as a translator. It maintains persistent MTProto connections to Telegram's core network on your behalf so you do not have to.\nWhen you run the Local Bot API Server, you are moving that translator to your own machine. Consequently, your local server must now handle the MTProto protocol.\nHere is why…"
prev_id = 1804
prev_title = ""
prev_body = "In #batumi, to buy a coffee, sometimes we talk in three #languages:\n#kartuli\n#english\n#russian"
views = 30
ids = [1805]

[[extra.related]]
path = "@/posts/2026-05-25-1786/index.md"
label = "Finally I forked #nixnote, and migrated from #qt5 to #qt6, by #l…"

[[extra.related]]
path = "@/posts/2026-05-21-1784/index.md"
label = "My new project built with #llm #codex #gpt-5.5 xhigh: #bot for #…"

[[extra.related]]
path = "@/posts/2026-06-19-1845/index.md"
label = "I did #kvantum/#qt6 theme, with #llm #gpt 5.5 xhigh, #love it, #…"

[[extra.related]]
path = "@/posts/2026-08-09-2139/index.md"
label = "Youta: моя лекция-демонстрация TUI аудио плеера для Ютуба, Яндек…"

[[extra.related]]
path = "@/posts/2026-05-18-1780/index.md"
label = "What a time - multiple #llm doing multiple projects, in parallel…"
+++

My yet another project: [@wikipedia\_unofficial\_bot](https://t.me/wikipedia_unofficial_bot)  

Built with {{ tag(t="llm") }} {{ tag(t="gpt") }} 5.5 xhigh  

[https://github.com/vitaly-zdanevich/bot_telegram_wikipedia](<https://github.com/vitaly-zdanevich/bot_telegram_wikipedia> "Telegram bot for Wikipedia search on AWS Lambda · 0 stars · Languages: Rust 95%, Shell 3%, HCL 2% · 44 commits · MIT · last push 2026-06-18")

👍 1
