+++
title = ""
date = 2026-10-09T09:33:23+00:00
description = "golang race package main import \"fmt\" func main() { var done bool var msg string go func() { msg = \"hello\" done = true }() for !done { } fmt.Println(msg) } Can you imagine - sometimes will be printed…"

[taxonomies]
days = ["2026-10-09"]
tags = ["golang", "race", "gemini"]

[extra]
id = 2199
day = "2026-10-09"
tg_url = "https://t.me/vitaly_zdanevich_chan/2199"
next_id = 2200
next_title = ""
next_body = "Это фанатский мод: от первого лица с оружием CS ходишь по картам и кампаниям Warcraft III, мир и квесты остаются варкрафтовскими.\nНужны Warcraft III 1.26a и файлы CS 1.6.\nРепозиторий:\nВидео автора мода:"
prev_id = 2198
prev_title = ""
prev_body = "#mem\n#voice\n#ai\n#cry\n#game\nBy m3m3centrall, source"
views = 1
ids = [2199]

[[extra.related]]
path = "@/posts/2026-10-09-2201/index.md"
label = "#golang What will be printed? package main import  \"fmt\"  func m…"
date = "2026-10-09"

[[extra.related]]
path = "@/posts/2026-06-12-1819/index.md"
label = "My another #userstyle: for #gemini, before and after"
date = "2026-06-12"

[[extra.related]]
path = "@/posts/2026-05-18-1779/index.md"
label = "--- /home/vitaly/.gemini/tmp/prime-world/chats -----------------…"
date = "2026-05-18"

[[extra.related]]
path = "@/posts/2025-09-08-657/index.md"
label = "#golang #utf8 const s = \"สวัสดี\" fmt.Println\"Len:\", lens // Len:…"
date = "2025-09-08"

[[extra.related]]
path = "@/posts/2025-09-02-655/index.md"
label = "#chatgpt 5: Escape shell $ as $$ inside Makefile recipes #gemini…"
date = "2025-09-02"
+++

{{ tag(t="golang") }}  
{{ tag(t="race") }}  

```
package main

import "fmt"

func main() {
  var done bool
  var msg string

  go func() {
    msg = "hello"
    done = true
  }()

  for !done {
  }
  fmt.Println(msg)
}
```

Can you imagine - **sometimes** will be printed an empty string.  

`-race` reports data race here.  

{{ tag(t="gemini") }} 3.8 flash high explanation:  

> Goroutines do not have separate memory blocks; all goroutines share the same virtual address space.  <br>  <br>An empty string can still be printed, but that happens because of compiler/hardware reordering and cache visibility, caused by a data race.  <br>  <br>  ### Why this happens  <br>  <br>  1. Shared Address Space:  <br>  Both variables (**done** and **msg**) reside in shared memory. Goroutines are lightweight threads managed by the Go runtime, not isolated processes.  <br>  <br>  2. No "Happens-Before" Guarantee:  <br>  According to the Go memory model, reads and writes across goroutines are only ordered when synchronized through mechanisms like channels, mutexes, or atomic operations. Without synchronization, the Go compiler and the CPU are free to:  <br>      • Reorder the writes **msg = "hello"** and **done = true**.  <br>      • Reorder the reads in main.  <br>      • Keep **msg** or **done** in CPU registers or store buffers without flushing to shared cache.  <br>  If **done = true** becomes visible to main before **msg = "hello"** does, **fmt.Println(msg)** prints **""**.  <br>  <br>  3. Infinite Loop Risk:  <br>  The code might also never terminate. The compiler can observe that **done** is not modified inside the body of **for !done {}** in main and hoist the read or keep **done** in a register:  <br>  <br>    // What the compiler is allowed to generate:  <br>    if !done {  <br>        for {  <br>        }  <br>    }  <br>  <br>  <br>  ### Detection  <br>  <br>  You can detect this issue using the Go race detector:  <br>  <br>    go run -race main.go  <br>  <br>  ### Proper Solution  <br>  <br>  Use synchronization tools such as channels, **sync.WaitGroup**, or atomic values to establish a happens-before relationship:  <br>  <br>  <br>    package main  <br>  <br>    import (  <br>       "fmt"  <br>      "sync"  <br>    )  <br>  <br>    // main coordinates goroutines using sync.WaitGroup to guarantee visibility.  <br>    func main() {  <br>        var (  <br>            wg  sync.WaitGroup  <br>            msg string  <br>        )  <br>  <br>        wg.Add(1)  <br>        go func() {  <br>            defer wg.Done()  <br>            msg = "hello"  <br>        }()  <br>  <br>        wg.Wait()  <br>        fmt.Println(msg)  <br>    }
