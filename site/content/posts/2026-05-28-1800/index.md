+++
title = ""
date = 2026-05-28T10:29:00+00:00
description = "remapped unused button on my laptop to git push, i3 command: bindsym XF86Launch1 exec --no-startup-id xdotool type \"git push\" && xdotool key Return How get to know the button code: on gentoo you need…"

[taxonomies]
days = ["2026-05-28"]
tags = ["remapped", "git", "push", "i3", "gentoo"]

[extra]
id = 1800
day = "2026-05-28"
tg_url = "https://t.me/vitaly_zdanevich_chan/1800"
next_id = 1802
next_title = ""
prev_id = 1799
prev_title = ""
views = 47
ids = [1800]
+++

{{ tag(t="remapped") }} unused button on my laptop to {{ tag(t="git") }} {{ tag(t="push") }}, {{ tag(t="i3") }} command:  

`bindsym XF86Launch1 exec --no-startup-id xdotool type "git push" && xdotool key Return`  

How get to know the button code: on {{ tag(t="gentoo") }} you need **x11-misc/xdotool**, run from root and press your any button. Here `XF86Launch1` is the button name.
