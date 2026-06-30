+++
title = ""
date = 2026-06-13T14:59:53+00:00
description = "shell productivity love my mg alias - clickable grep in kitty - opens file and line in Vim: Grep, click to link - open in Vim, exact line mg() { kitty +kitten hyperlinkedgrep --smart-case -C 9 \"$@\" }…"

[taxonomies]
days = ["2026-06-13"]
tags = ["shell", "productivity", "love", "grep", "kitty"]

[extra]
id = 1823
day = "2026-06-13"
tg_url = "https://t.me/vitaly_zdanevich_chan/1823"
next_id = 1824
next_title = ""
prev_id = 1822
prev_title = ""
views = 15
ids = [1823]
+++

{{ tag(t="shell") }}
{{ tag(t="productivity") }}
{{ tag(t="love") }} my `mg` alias - clickable {{ tag(t="grep") }} in {{ tag(t="kitty") }} - opens file and line in Vim:

```
# Grep, click to link - open in Vim, exact line
mg() {
  kitty +kitten hyperlinked_grep --smart-case -C 9 "$@"
}
```

`-C 9` is the context - to have a few lines before and after.

For this, also you need to have in `~/.config/kitty/open-actions.conf`:

```
protocol file
fragment_matches [0-9]+
action launch --type=overlay -- vim +$FRAGMENT -- $FILE_PATH
```

<https://sw.kovidgoyal.net/kitty/kittens/hyperlinked_grep/>
