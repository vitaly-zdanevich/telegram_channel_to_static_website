+++
title = ""
date = 2026-02-10T16:56:36+00:00
description = "My new bash alias: one function to go to the next folder (like from 2025 to 2026, from aaa to bbb) and the second one to cd to prev: cdn() { local cur next cur=\"$(basename \"$PWD\")\" next=\"$( find ..…"

[taxonomies]
days = ["2026-02-10"]
tags = ["bash", "cd"]

[extra]
id = 1103
day = "2026-02-10"
tg_url = "https://t.me/vitaly_zdanevich_chan/1103"
next_id = 1104
next_title = ""
prev_id = 1102
prev_title = ""
views = 20
ids = [1103]
+++

My new {{ tag(t="bash") }} alias: one function to go to the next folder (like from 2025 to 2026, from aaa to bbb) and the second one to {{ tag(t="cd") }} to prev:  

```
cdn() {
  local cur next
  cur="$(basename "$PWD")"

  next="$(
    find .. -mindepth 1 -maxdepth 1 -type d -printf '%f\n' \
    | sort \
    | awk -v cur="$cur" '$1>cur{print; exit}'
  )"

  if [[ -z "$next" ]]; then
    echo "no next folder"
    return 1
  fi

  cd "../$next"
}

cdp() {
  cur="$(basename "$PWD")"

  prev="$(
    find .. -mindepth 1 -maxdepth 1 -type d -printf '%f\n' \
    | sort \
    | awk -v cur="$cur" '$1<cur{p=$1} END{print p}'
  )"

  if [[ -z "$prev" ]]; then
    echo "no previous folder"
    return 1
  fi

  cd "../$prev"
}
```
