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
next_body = "# cd to a first sorted folder, exclude hidden\nfirst() {\nlocal first\nfirst=\"$(\nfind . -mindepth 1 -maxdepth 1 -type d ! -name '.' -printf '%fn'\n| sort\n| head -n 1\n)\"\nif [[ -z \"$first\" ]]; then\necho \"no folders\"\nreturn 1\nfi\ncd \"./$first\"\n}"
prev_id = 1102
prev_title = ""
prev_body = "How to make #ubuntu folder for #chroot\nwget\n# Only 34 MB\nwget\nsha256sum -c SHA256SUMS 2/dev/null\nmkdir ubuntu-base\ntar -xzf ubuntu-base-25.10-base-amd64.tar.gz -C ubuntu-base\ncd ubuntu-base/\ncp /etc/resolv.conf etc/resolv.conf\nCreate file inside:\nmount --bind /dev dev\nmount --bind /proc proc\nmount --bind /sys sys\nmount --bind /run run\nchroot . /bin/bash\nand run from root (because requires CAPSYSCHROOT)"
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
