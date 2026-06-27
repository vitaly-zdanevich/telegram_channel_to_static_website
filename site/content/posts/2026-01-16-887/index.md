+++
title = "commons: count uploads from a specific user for a period of time, python"
date = 2026-01-16T19:32:14+00:00
description = "commons: count uploads from a specific user for a period of time, python: import requests user = 'Globustut' start = '2026-01-20T00:00:00Z' newer end = '2026-01-01T00:00:00Z' older params = {…"

[taxonomies]
tags = ["commons", "count", "python"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/887"
next_id = 888
next_title = "commons: red category links with one or more files from a specific user"
prev_id = 886
prev_title = "commons: files from a specific user in a non-existing categories, sql"
views = 11
ids = [887]
+++

{{ tag(t="commons") }}: {{ tag(t="count") }} uploads from a specific user for a period of time, {{ tag(t="python") }}:

```
import requests

user = 'Globustut'
start = '2026-01-20T00:00:00Z'  # newer
end   = '2026-01-01T00:00:00Z'  # older

params = {
    'action': 'query',
    'format': 'json',
    'list': 'usercontribs',
    'ucuser': user,
    'ucnamespace': '6',
    'ucshow': 'new',
    'ucstart': start,
    'ucend': end,
    'uclimit': 'max',
}
headers = {'User-Agent': 'commons-upload-count/1.0'}

total = 0
s = requests.Session()
while True:
    data = s.get('https://commons.wikimedia.org/w/api.php', params=params, headers=headers, timeout=30).json()
    total += len(data.get('query', {}).get('usercontribs', []))
    if 'continue' not in data:
        break
    print('.', end='')
    params.update(data['continue'])

print(total)
```

<https://quarry.wmcloud.org/query/100891>
