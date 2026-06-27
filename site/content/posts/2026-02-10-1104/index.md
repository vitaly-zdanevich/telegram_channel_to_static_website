+++
title = "cd to a first sorted folder, exclude hidden"
date = 2026-02-10T19:31:40+00:00
description = "first() { local first first=\"$( find . -mindepth 1 -maxdepth 1 -type d ! -name '.' -printf '%fn' | sort | head -n 1 )\" if [[ -z \"$first\" ]]; then echo \"no folders\" return 1 fi cd \"./$first\" }"

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1104"
next_id = 1105
next_title = "If you cannot upload your big pdf - you can extract all images from it, with original quality"
prev_id = 1103
prev_title = "My new bash alias: one function to go to the next folder (like from 2025 to 2026, from aaa to bbb) and the second one to cd to prev"
views = 23
ids = [1104]
+++

```
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
