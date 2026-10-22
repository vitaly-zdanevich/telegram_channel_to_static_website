+++
title = ""
date = 2026-02-10T16:56:36+00:00
description = "My new bash alias: one function to go to the next folder (like from 2025 to 2026, from aaa to bbb) and the second one to cd to prev: cdn() { local cur next cur=\"$(basename \"$PWD\")\" next=\"$( find ..…"

[taxonomies]
days = ["2026-02-10"]
tags = ["bash", "cd"]

[extra]
discussions_html = "<section class='discussions' aria-labelledby='comments-heading'><h2 id='comments-heading'>Comments (3)</h2><ol class='discussion-comments'><li id='comment-2033173701-1341'><div class='comment-meta'><span class='comment-author'>Mrtko</span> · <a href='https://t.me/vitaly_zdanevich_chan/1103?comment=1341'><time datetime='2026-02-10T19:00:47+00:00'>2026-02-10 19:00 +00:00</time></a></div><div class='comment-body'>What&#39;s a use case?</div></li><li id='comment-2033173701-1342'><div class='comment-meta'><span class='comment-author'>Unknown author</span> · <a href='https://t.me/vitaly_zdanevich_chan/1103?comment=1342'><time datetime='2026-02-10T19:31:11+00:00'>2026-02-10 19:31 +00:00</time></a></div><div class='comment-body'>To simplify\n\n<pre>cd ..\nls -l\n&lt;thinking looking for the next folder name&gt;\ncd &lt;that next folder name&gt;</pre></div></li><li id='comment-2033173701-1344'><div class='comment-meta'><span class='comment-author'>Mrtko</span> · <a href='https://t.me/vitaly_zdanevich_chan/1103?comment=1344'><time datetime='2026-02-10T20:02:51+00:00'>2026-02-10 20:02 +00:00</time></a></div><div class='comment-body'>I got the flow but didn&#39;t figure out what one can use it for. I&#39;m curious because Bash aliases are one of my favorite tools</div></li></ol><p><a href='https://t.me/vitaly_zdanevich_chan/1103?comment=0'>Reply on Telegram</a></p></section>"
id = 1103
day = "2026-02-10"
tg_url = "https://t.me/vitaly_zdanevich_chan/1103"
next_id = 1104
next_title = ""
next_body = "# cd to a first sorted folder, exclude hidden\nfirst() {\nlocal first\nfirst=\"$(\nfind . -mindepth 1 -maxdepth 1 -type d ! -name '.' -printf '%fn'\n| sort\n| head -n 1\n)\"\nif [[ -z \"$first\" ]]; then\necho \"no folders\"\nreturn 1\nfi\ncd \"./$first\"\n}"
prev_id = 1102
prev_title = ""
prev_body = "How to make #ubuntu folder for #chroot\nwget\n# Only 34 MB\nwget\nsha256sum -c SHA256SUMS 2/dev/null\nmkdir ubuntu-base\ntar -xzf ubuntu-base-25.10-base-amd64.tar.gz -C ubuntu-base\ncd ubuntu-base/\ncp /etc/resolv.conf etc/resolv.conf\nCreate file inside:\nmount --bind /dev dev\nmount --bind /proc proc\nmount --bind /sys sys\nmount --bind /run run\nchroot . /bin/bash\nand run from root (because requires CAPSYSCHROOT)"
views = 21
ids = [1103]

[[extra.related]]
path = "@/posts/2026-04-25-1682/index.md"
label = "My new great #bash #alias actually a #hotkey for faster #cd # Li…"
date = "2026-04-25"

[[extra.related]]
path = "@/posts/2026-03-05-1336/index.md"
label = "...one of my #bash aliases: to count files here: c { ls -1  wc -…"
date = "2026-03-05"

[[extra.related]]
path = "@/posts/2026-06-24-1864/index.md"
label = "And another #bash #alias: # Better word movement: treat aaabbbcc…"
date = "2026-06-24"

[[extra.related]]
path = "@/posts/2026-06-24-1863/index.md"
label = "My new #bash #alias bind '\"ej\": \"!#:$ e^\"' # Alt J repeats prev…"
date = "2026-06-24"

[[extra.related]]
path = "@/posts/2026-03-05-1334/index.md"
label = "#bash I love #cli, scripts, and sometimes I want my script to ac…"
date = "2026-03-05"
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
