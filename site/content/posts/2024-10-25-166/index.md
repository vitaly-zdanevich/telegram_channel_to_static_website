+++
title = ""
date = 2024-10-25T22:43:46+00:00
description = "List mp3 with bitrate and sorting while read i; do echo \"$(ffprobe -i \"$i\" 2&1 | egrep -o 'bitrate: [0-9]{1,3} .{4}' | cut -d' ' -f2-3)\" \"$i\"; done < <(find . -type f -printf '%Pn' ( -iname .ogg -o…"

[taxonomies]
days = ["2024-10-25"]

[extra]
id = 166
day = "2024-10-25"
tg_url = "https://t.me/vitaly_zdanevich_chan/166"
next_id = 167
next_title = ""
next_body = "Source:\nAuthor: Oleg Paschenko"
prev_id = 165
prev_title = ""
prev_body = "#rice"
views = 31
ids = [166]
+++

List mp3 with bitrate and sorting  

```
while read i; do echo "$(ffprobe -i "$i" 2>&1 | egrep -o 'bitrate: [0-9]{1,3} .{4}' | cut -d' ' -f2-3)" "$i"; done < <(find . -type f -printf '%P\n' \( -iname \*.ogg -o -iname \*.mp3 \)) | sort -n -k1,1
```

```
128 kb/s aaa.mp3
128 kb/s bbb.mp3
128 kb/s ccc.mp3
256 kb/s xxx.mp3
256 kb/s yyy.mp3
256 kb/s zzz.mp3
```

Based on <https://www.linuxquestions.org/questions/linux-newbie-8/how-to-list-bitrate-of-all-my-mp3%27s-on-command-line-4175601321/#post5680865>  

Published to <https://gitlab.com/vitaly-zdanevich/ffprobe-wrapper-lister-for-mp3>
