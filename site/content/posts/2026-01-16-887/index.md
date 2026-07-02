+++
title = ""
date = 2026-01-16T19:32:14+00:00
description = "commons: count uploads from a specific user for a period of time, python: import requests user = 'Globustut' start = '2026-01-20T00:00:00Z' newer end = '2026-01-01T00:00:00Z' older params = {…"

[taxonomies]
days = ["2026-01-16"]
tags = ["commons", "count", "python"]

[extra]
id = 887
day = "2026-01-16"
tg_url = "https://t.me/vitaly_zdanevich_chan/887"
next_id = 888
next_title = ""
next_body = "#sql\n#quarry\n#globustut\n#commons: red category links with one or more files from a specific user\nSELECT\nCONCAT(' REPLACE(cl.clto, ' ', '')) AS categoryurl,\nCOUNT() AS filecount\nFROM page p\nJOIN image i ON i.imgname = p.pagetitle\nJOIN actor a ON a.actorid = i.imgactor\nJOIN categorylinks cl ON cl.clfrom = p.pageid\nLEFT JOIN page c\nON c.pagetitle = cl.clto\nAND c.pagenamespace = 14\nWHERE p.pagenamespace = 6\nAND a.actorname = 'Globustut'\nAND c.pageid IS NULL\nGROUP BY cl.clto\nORDER BY filecount DESC, cl.clto"
prev_id = 886
prev_title = ""
prev_body = "#sql\n#quarry\n#globustut\n#commons: files from a specific user in a non-existing categories, #sql:\nsql\nSELECT\nimgname,\nclto AS missingcategory\nFROM image\nJOIN actor ON actorid = imgactor\nJOIN page ON pagenamespace = 6 AND pagetitle = imgname\nJOIN categorylinks ON clfrom = pageid\nLEFT JOIN page AS cat\nON cat.pagenamespace = 14\nAND cat.pagetitle = clto\nWHERE actorname = 'Globustut'\nAND cat.pageid IS NULL\nORDER BY imgname;"
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
