+++
title = ""
date = 2026-05-06T20:05:38+00:00
description = "This is how in go we remove an element from a collection func rm(i int, lists[]ListNode) []ListNode { return append(lists[:i], lists[i+1:]...) } Usually its a library call."

[taxonomies]
days = ["2026-05-06"]
tags = ["go"]

[extra]
id = 1750
day = "2026-05-06"
tg_url = "https://t.me/vitaly_zdanevich_chan/1750"
next_id = 1751
next_title = ""
prev_id = 1749
prev_title = ""
views = 27
ids = [1750]
+++

This is how in {{ tag(t="go") }} we remove an element from a collection  

```
func rm(i int, lists[]*ListNode) []*ListNode {
    return append(lists[:i], lists[i+1:]...)
}
```

Usually its a library call.
