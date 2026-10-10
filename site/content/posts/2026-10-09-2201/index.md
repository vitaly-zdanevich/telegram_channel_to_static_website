+++
title = ""
date = 2026-10-09T10:20:24+00:00
description = "golang What will be printed? package main import ( \"fmt\" ) func main() { foo := 1 defer fmt.Println(foo) Up(&foo) defer fmt.Println(foo) } func Up(bar int) { bar = bar + 1 } Answer: 2 1 And now -…"

[taxonomies]
days = ["2026-10-09"]
tags = ["golang", "defer"]

[extra]
id = 2201
day = "2026-10-09"
tg_url = "https://t.me/vitaly_zdanevich_chan/2201"
prev_id = 2200
prev_title = ""
prev_body = "Это фанатский мод: от первого лица с оружием CS ходишь по картам и кампаниям Warcraft III, мир и квесты остаются варкрафтовскими.\nНужны Warcraft III 1.26a и файлы CS 1.6.\nРепозиторий:\nВидео автора мода:"
views = 2
ids = [2201]

[[extra.related]]
path = "@/posts/2025-09-08-657/index.md"
label = "#golang #utf8 const s = \"สวัสดี\" fmt.Println\"Len:\", lens // Len:…"
date = "2025-09-08"

[[extra.related]]
path = "@/posts/2026-10-09-2199/index.md"
label = "#golang #race package main import \"fmt\" func main { var done boo…"
date = "2026-10-09"

[[extra.related]]
path = "@/posts/2026-09-19-2164/index.md"
label = "#webdesign #mascot #golang From"
date = "2026-09-19"

[[extra.related]]
path = "@/posts/2026-05-03-1732/index.md"
label = "#gentoo #golang #bootstrap"
date = "2026-05-03"
+++

{{ tag(t="golang") }}  

What will be printed?  

```
package main

import (
  "fmt"
)

func main() {
  foo := 1
  defer fmt.Println(foo)
  Up(&foo)
  defer fmt.Println(foo)
}

func Up(bar *int) {
  *bar = *bar + 1
}
```

Answer:  
<span class="spoiler">2  
1</span>  

And now - wrapped {{ tag(t="defer") }} with a func:  

```
package main

import (
  "fmt"
)

func main() {
  foo := 1
  defer func() {
    fmt.Println(foo)
  }()
  Up(&foo)
  defer func() {
    fmt.Println(foo)
  }()
}

func Up(bar *int) {
  *bar = *bar + 1
}
```

Answer:  
<span class="spoiler">2  
2</span>
