+++
title = ""
date = 2026-02-10T19:31:40+00:00
description = "cd to a first sorted folder, exclude hidden first() { local first first=\"$( find . -mindepth 1 -maxdepth 1 -type d ! -name '.' -printf '%fn' | sort | head -n 1 )\" if [[ -z \"$first\" ]]; then echo \"no…"

[taxonomies]
days = ["2026-02-10"]

[extra]
id = 1104
day = "2026-02-10"
tg_url = "https://t.me/vitaly_zdanevich_chan/1104"
next_id = 1105
next_title = ""
prev_id = 1103
prev_title = ""
views = 23
ids = [1104]
+++

```
# cd to a first sorted folder, exclude hidden
first() {
  local first
  first="$(
    find . -mindepth 1 -maxdepth 1 -type d ! -name '.*' -printf '%f\n' \
    | sort \
    | head -n 1
  )"

  if [[ -z "$first" ]]; then
    echo "no folders"
    return 1
  fi

  cd "./$first"
}
```
