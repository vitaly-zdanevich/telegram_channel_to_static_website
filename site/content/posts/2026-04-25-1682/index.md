+++
title = ""
date = 2026-04-25T03:33:17+00:00
description = "My new great bash alias (actually a hotkey) for faster cd Like cd, but accept a path with spaces - so no quotation is needed How to use it: paste/type a path and press Ctrl-g bind \"\"C-g\": \"C-acd…"

[taxonomies]
days = ["2026-04-25"]
tags = ["bash", "alias", "hotkey", "cd"]

[extra]
id = 1682
day = "2026-04-25"
tg_url = "https://t.me/vitaly_zdanevich_chan/1682"
next_id = 1683
next_title = ""
prev_id = 1681
prev_title = ""
views = 17
ids = [1682]
+++

My new great {{ tag(t="bash") }} {{ tag(t="alias") }} (actually a {{ tag(t="hotkey") }}) for faster {{ tag(t="cd") }}  

```
# Like cd, but accept a path with spaces - so no quotation is needed
# How to use it: paste/type a path and press Ctrl-g
bind "\"\C-g\": \"\C-acd '\C-e'\C-j\""
# How it works: it executes multiple commands:
# Ctrl-a to go to the line start
# Types cd '
# Ctrl-e to go to the line end
# Add single quote
# Ctrl-j hits the Enter
# =====
# I tried an alias/function
# `cd "$*"`
# and
# `cd "$1"`
# but got error when path contains `(`
```
