+++
title = ""
date = 2025-09-29T05:52:05+00:00
description = "patch for telegram for wide messages --- a/Telegram/SourceFiles/ui/chat/chat.style 2024-08-02 09:26:52.899323105 +0700 +++ b/Telegram/SourceFiles/ui/chat/chat.style 2024-08-02 09:27:23.226355858…"

[taxonomies]
days = ["2025-09-29"]
tags = ["patch", "telegram"]

[extra]
id = 684
day = "2025-09-29"
tg_url = "https://t.me/vitaly_zdanevich_chan/684"
og_image = "5391335068301129921_1255268014_456259777.jpg"
next_id = 685
next_title = ""
next_body = "#fear\n#airplane\nSource"
prev_id = 683
prev_title = ""
prev_body = "Love this #logo\n#head\n#psy"
views = 22
ids = [684]

[[extra.related]]
path = "@/posts/2026-03-17-1491/index.md"
label = "#telegram added a feature request Add option to cache/prefetch a…"
date = "2026-03-17"

[[extra.related]]
path = "@/posts/2025-09-20-674/index.md"
label = "#telegram with wide messages"
date = "2025-09-20"

[[extra.related]]
path = "@/posts/2025-08-05-614/index.md"
label = "#telegram bot that sends to email, its mean to #evernote too! @s…"
date = "2025-08-05"

[[extra.related]]
path = "@/posts/2026-06-08-1806/index.md"
label = "Wow, about #telegram bots: you can bypass 50 MB response limit -…"
date = "2026-06-08"

[[extra.related]]
path = "@/posts/2025-03-24-442/index.md"
label = "wow in #telegram we have a #crypto #wallet, and users can send m…"
date = "2025-03-24"
+++

{{ tag(t="patch") }} for {{ tag(t="telegram") }} for wide messages  

```
--- a/Telegram/SourceFiles/ui/chat/chat.style  2024-08-02 09:26:52.899323105 +0700
+++ b/Telegram/SourceFiles/ui/chat/chat.style  2024-08-02 09:27:23.226355858 +0700
@@ -11,7 +11,7 @@ using "ui/widgets/widgets.style";
 using "ui/menu_icons.style";
 using "chat_helpers/chat_helpers.style"; // GroupCallUserpics

-msgMaxWidth: 430px;
+msgMaxWidth: 2430px;
 msgFont: font(fsize);
 msgNameFont: semiboldFont;
 msgNameStyle: semiboldTextStyle;
```

[https://github.com/msva/mva-overlay/blob/master/net-im/telegram-desktop/files/patches/0/conditional/tdesktop_patches_wide-baloons/style.patch](<https://github.com/msva/mva-overlay/blob/master/net-im/telegram-desktop/files/patches/0/conditional/tdesktop_patches_wide-baloons/style.patch> "mva's sandbox overlay · 61 stars · Languages: Shell 97%, Lua 1%, Makefile 1% · 3032 commits · 27 forks · 3 open issues/PRs · last push 2026-08-04")

{{ img(src="5391335068301129921_1255268014_456259777.jpg") }}
