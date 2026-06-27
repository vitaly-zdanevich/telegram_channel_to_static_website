+++
title = "...and another useful bash alias to remove the text inside single quotes by esc-k"
date = 2026-04-25T03:52:22+00:00
description = "...and another useful bash alias to remove the text inside single quotes by esc-k: Clear the content between single quotes while preserving the quotes clearinsidequotes() { local…"

[taxonomies]
tags = ["bash", "alias", "remove", "quotes"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1683"
next_id = 1684
next_title = "quote book ночь_в_лиссабоне ремарк"
prev_id = 1682
prev_title = "My new great bash alias (actually a hotkey) for faster cd"
views = 19
ids = [1683]
+++

...and another useful {{ tag(t="bash") }} {{ tag(t="alias") }} to {{ tag(t="remove") }} the text inside single {{ tag(t="quotes") }} by `esc-k`:

```
# Clear the content between single quotes while preserving the quotes
clear_inside_quotes() {
  local line="${READLINE_LINE}"

  # Only run if the line contains at least two single quotes
  if [[ "${line}" == *"'"*"'"* ]]; then

    # prefix: everything before the first quote
    local prefix="${line%%\'*}"

    # suffix: everything after the last quote
    local suffix="${line##*\'}"

    # Reconstruct the line with empty quotes
    READLINE_LINE="${prefix}''${suffix}"

    # Move the cursor back inside the empty quotes
    READLINE_POINT=$((${#prefix} + 1))
  fi
}
# Bind the function to Alt-k (Escape + k)
bind -x '"\ek": clear_inside_quotes'
```
