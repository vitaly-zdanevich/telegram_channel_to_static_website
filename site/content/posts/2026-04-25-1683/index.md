+++
title = ""
date = 2026-04-25T03:52:22+00:00
description = "...and another useful bash alias to remove the text inside single quotes by esc-k: Clear the content between single quotes while preserving the quotes clearinsidequotes() { local…"

[taxonomies]
days = ["2026-04-25"]
tags = ["bash", "alias", "remove", "quotes"]

[extra]
id = 1683
day = "2026-04-25"
tg_url = "https://t.me/vitaly_zdanevich_chan/1683"
next_id = 1684
next_title = ""
next_body = "#quote\n#book\n#ночьвлиссабоне\n#ремарк\n– Я тоже. В ту пору я вконец пал духом. Накануне Мюнхенского соглашения. Агония страха. Я еще автоматически прятался и защищался, но уже поставил точку. Будет война, придут немцы и заберут меня. Такова моя судьба. Я смирился.\nЯ кивнул:\n– Это было время самоубийств. Странно, когда через полтора года в самом деле пришли немцы, самоубийства случались реже.\n– Потом заключили Мюнхенские соглашения, – сказал Шварц. – Тогда, осенью тридцать восьмого, нам вдруг снова подарили жизнь! Она казалась такой легкой, что люди забывали об осторожности. В Париже даже второй раз зацвели каштаны, помните? Я стал настолько легкомыслен, что чувствовал себя человеком и, увы, так себя и вел. Полиция схватила меня и по причине повторного недозволенного въезда засадила на месяц под арест. Потом началась старая игра: под Базелем швейцарцы выдворили меня назад, за границу, французы в другом месте опять выпихнули в Швейцарию, арестовали… Ну, вы же знаете эти шахматы с…"
prev_id = 1682
prev_title = ""
prev_body = "My new great #bash #alias (actually a #hotkey) for faster #cd\n# Like cd, but accept a path with spaces - so no quotation is needed\n# How to use it: paste/type a path and press Ctrl-g\nbind \"\"C-g\": \"C-acd 'C-e'C-j\"\"\n# How it works: it executes multiple commands:\n# Ctrl-a to go to the line start\n# Types cd '\n# Ctrl-e to go to the line end\n# Add single quote\n# Ctrl-j hits the Enter\n# =====\n# I tried an alias/function\n# cd \"$\"\n# and\n# cd \"$1\"\n# but got error when path contains ("
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
