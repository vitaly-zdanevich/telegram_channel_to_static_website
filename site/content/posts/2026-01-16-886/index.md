+++
title = ""
date = 2026-01-16T19:27:30+00:00
description = "sql quarry globustut commons: files from a specific user in a non-existing categories, sql: sql SELECT imgname, clto AS missingcategory FROM image JOIN actor ON actorid = imgactor JOIN page ON…"

[taxonomies]
days = ["2026-01-16"]
tags = ["sql", "quarry", "globustut", "commons"]

[extra]
id = 886
day = "2026-01-16"
tg_url = "https://t.me/vitaly_zdanevich_chan/886"
og_image = "5429641422056394562_1264186907_460001090.jpg"
next_id = 887
next_title = ""
next_body = "#commons: #count uploads from a specific user for a period of time, #python:\nimport requests\nuser = 'Globustut'\nstart = '2026-01-20T00:00:00Z' # newer\nend = '2026-01-01T00:00:00Z' # older\nparams = {\n'action': 'query',\n'format': 'json',\n'list': 'usercontribs',\n'ucuser': user,\n'ucnamespace': '6',\n'ucshow': 'new',\n'ucstart': start,\n'ucend': end,\n'uclimit': 'max',\n}\nheaders = {'User-Agent': 'commons-upload-count/1.0'}\ntotal = 0\ns = requests.Session()\nwhile True:\ndata = s.get(' params=params, headers=headers, timeout=30).json()\ntotal += len(data.get('query', {}).get('usercontribs', []))\nif 'continue' not in data:\nbreak\nprint('.', end='')\nparams.update(data['continue'])\nprint(total)"
prev_id = 879
prev_title = ""
prev_body = "#belarus\n#architecture\n#castle\n#globustut\nKosava Castle\nSource"
views = 11
ids = [886]

[[extra.related]]
path = "@/posts/2026-01-16-888/index.md"
label = "#sql #quarry #globustut #commons: red category links with one or…"
date = "2026-01-16"

[[extra.related]]
path = "@/posts/2026-02-22-1123/index.md"
label = "#commons"
date = "2026-02-22"

[[extra.related]]
path = "@/posts/2026-02-05-1093/index.md"
label = "#commons My account is big, my account is very big"
date = "2026-02-05"

[[extra.related]]
path = "@/posts/2025-06-19-584/index.md"
label = "#commons TODO list"
date = "2025-06-19"

[[extra.related]]
path = "@/posts/2026-03-02-1306/index.md"
label = "Magic that I can say #codex to download all scan - and I get it,…"
date = "2026-03-02"
+++

{{ tag(t="sql") }}  
{{ tag(t="quarry") }}  
{{ tag(t="globustut") }}  
{{ tag(t="commons") }}: files from a specific user in a non-existing categories, {{ tag(t="sql") }}:  

```
sql
SELECT
    img_name,
    cl_to AS missing_category
  FROM image
  JOIN actor ON actor_id = img_actor
  JOIN page ON page_namespace = 6 AND page_title = img_name
  JOIN categorylinks ON cl_from = page_id
  LEFT JOIN page AS cat
    ON cat.page_namespace = 14
   AND cat.page_title = cl_to
  WHERE actor_name = 'Globustut'
    AND cat.page_id IS NULL
  ORDER BY img_name;
```

<https://quarry.wmcloud.org/query/101097>

{{ img(src="5429641422056394562_1264186907_460001090.jpg") }}
