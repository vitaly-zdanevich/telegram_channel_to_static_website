+++
title = ""
date = 2026-10-09T09:58:02+00:00
description = "Это фанатский мод: от первого лица с оружием CS ходишь по картам и кампаниям Warcraft III, мир и квесты остаются варкрафтовскими. Нужны Warcraft III 1.26a и файлы CS 1.6. Репозиторий: Видео автора…"

[taxonomies]
days = ["2026-10-09"]

[extra]
id = 2200
day = "2026-10-09"
tg_url = "https://t.me/vitaly_zdanevich_chan/2200"
next_id = 2201
next_title = ""
next_body = "#golang\nWhat will be printed?\npackage main\nimport (\n\"fmt\"\n)\nfunc main() {\nfoo := 1\ndefer fmt.Println(foo)\nUp(&foo)\ndefer fmt.Println(foo)\n}\nfunc Up(bar int) {\nbar = bar + 1\n}\nAnswer:\n2\n1\nAnd now - wrapped #defer with a func:\npackage main\nimport (\n\"fmt\"\n)\nfunc main() {\nfoo := 1\ndefer func() {\nfmt.Println(foo)\n}()\nUp(&foo)\ndefer func() {\nfmt.Println(foo)\n}()\n}\nfunc Up(bar int) {\nbar = bar + 1\n}\nAnswer:\n2\n2"
prev_id = 2199
prev_title = ""
prev_body = "#golang\n#race\npackage main\nimport \"fmt\"\nfunc main() {\nvar done bool\nvar msg string\ngo func() {\nmsg = \"hello\"\ndone = true\n}()\nfor !done {\n}\nfmt.Println(msg)\n}\nCan you imagine - sometimes will be printed an empty string.\n-race reports data race here.\n#gemini 3.8 flash high explanation:\nGoroutines do not have separate memory blocks; all goroutines share the same virtual address space.\nAn empty string can still be printed, but that happens because of compiler/hardware reordering and cache visibility, caused by a data race.\n### Why this happens\n1. Shared Address Space:\nBoth variables (done and msg) reside in shared memory. Goroutines are lightweight threads managed by the Go runtime, not isolated processes.\n2. No \"Happens-Before\" Guarantee:\nAccording to the Go memory model, reads and writes across goroutines are only ordered when synchronized through mechanisms like channels, mutexes, or atomic operations. Without synchronization, the Go compiler and the CPU are free to:\n• Reorder the…"
views = 1
forwarded_from = "FroZi"
forwarded_from_url = "https://t.me/aiFroZi/1310"
ids = [2200]
+++

Это фанатский мод: от первого лица с оружием CS ходишь по картам и кампаниям Warcraft III, мир и квесты остаются варкрафтовскими.  

Нужны Warcraft III 1.26a и файлы CS 1.6.  

Репозиторий: [https://github.com/YazgulDev/warcraft-cs](<https://github.com/YazgulDev/warcraft-cs> "Counter-Strike-style offline FPS for Warcraft III 1.26a: RoC/TFT, gold buy menu, squads, configurable weapons and CS audio · 44 stars · Languages: C++ 60%, PowerShell 17%, C# 15% · 92 commits · 4 forks · 2 open issues/PRs · Apache-2.0 · last push 2026-10-09")  

Видео автора мода: [https://www.youtube.com/watch?v=hW0TytuwpRA](<https://www.youtube.com/watch?v=hW0TytuwpRA> "Playing Counter Strike 1.6 in Warcraft 3 mod")

{{ youtube(id="hW0TytuwpRA") }}
