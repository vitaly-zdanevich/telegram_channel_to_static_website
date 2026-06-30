+++
title = ""
date = 2026-05-12T12:45:18+00:00
description = "heap lt Wow in leetcode we can patch classes: ListNode.lt = lambda self, other: self.val Optional[ListNode]: heap = [] for node in lists: if node: heapq.heappush(heap, node) head: List[ListNode] =…"

[taxonomies]
days = ["2026-05-12"]
tags = ["heap", "lt", "leetcode", "patch"]

[extra]
id = 1753
day = "2026-05-12"
tg_url = "https://t.me/vitaly_zdanevich_chan/1753"
og_image = "01.jpg"
next_id = 1755
next_title = ""
prev_id = 1752
prev_title = ""
views = 21
ids = [1753]
+++

{{ tag(t="heap") }}
{{ tag(t="lt") }}

Wow in {{ tag(t="leetcode") }} we can {{ tag(t="patch") }} classes:

```
ListNode.__lt__ = lambda self, other: self.val < other.val
```

This is good for simpler support of [heap](https://docs.python.org/3/library/heapq.html) - by providing the object only - without the index and value.

Full:

```
import heapq

# Definition for singly-linked list.
# class ListNode:
#     def __init__(self, val=0, next=None):
#         self.val = val
#         self.next = next

ListNode.__lt__ = lambda self, other: self.val < other.val

class Solution:
    def mergeKLists(self, lists: List[Optional[ListNode]]) -> Optional[ListNode]:
        heap = []

        for node in lists:
            if node:
                heapq.heappush(heap, node)

        head: List[ListNode] = ListNode()
        tail = head

        while heap:
            node = heapq.heappop(heap)
            tail.next = node
            tail = node

            if node.next:
                heapq.heappush(heap, node.next)

        return head.next
```

[23. Merge k Sorted Lists](https://leetcode.com/problems/merge-k-sorted-lists)

![](01.jpg)

![](02.jpg)
