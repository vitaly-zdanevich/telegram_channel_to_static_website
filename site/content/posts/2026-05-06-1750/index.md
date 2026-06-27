+++
title = "This is how in go we remove an element from a collection"
date = 2026-05-06T20:05:38+00:00
description = "This is how in go we remove an element from a collection func rm(i int, lists[]ListNode) []ListNode { return append(lists[:i], lists[i+1:]...) } Usually its a library call."

[taxonomies]
tags = ["go"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1750"
next_id = 1751
next_title = "7мая2026 (чт) 21.30-01:00 айтишная посиделка в Still Young Bar"
prev_id = 1749
prev_title = "leetcode validation"
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
