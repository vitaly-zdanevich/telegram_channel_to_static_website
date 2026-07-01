+++
title = ""
date = 2026-04-11T10:18:23+00:00
description = "I wrote, with ai, a nice script to find top biggest folders - that has no other subfolders (leaf folders): find . -type d -links 2 -exec du -sh {} + | sort -hr | head Output example: 6.3G ./ДА…"

[taxonomies]
days = ["2026-04-11"]
tags = ["ai", "script", "find", "biggest", "folders", "leaf", "hardlink"]

[extra]
id = 1612
day = "2026-04-11"
tg_url = "https://t.me/vitaly_zdanevich_chan/1612"
next_id = 1613
next_title = ""
prev_id = 1602
prev_title = ""
views = 18
ids = [1612]
+++

I wrote, with {{ tag(t="ai") }}, a nice {{ tag(t="script") }} to {{ tag(t="find") }} top {{ tag(t="biggest") }} {{ tag(t="folders") }} - that has no other subfolders ({{ tag(t="leaf") }} folders):  

```
find . -type d -links 2 -exec du -sh {} + | sort -hr | head
```

Output example:  

> 6.3G  ./ДА Житомирської області/01 Ф - фонди дорадянського періоду/0118/010118-14/010118-14-00018  <br>5.3G  ./ЦДІАЛ/0080/010020-10-00088  <br>4.8G  ./ДА Закарпатської області/Перепис Закарпаття 1921 року/024/Beregszasz  <br>4.3G  ./ДА Закарпатської області/Перепис Закарпаття 1921 року/054  <br>4.3G  ./ДА Закарпатської області/Перепис Закарпаття 1921 року/045  <br>4.1G  ./ДА Закарпатської області/Перепис Закарпаття 1921 року/026/Beregszasz  <br>3.8G  ./ДА Закарпатської області/Перепис Закарпаття 1921 року/044  <br>3.5G  ./ДА Харківської області/01/0031/010031-141-00523  <br>3.4G  ./ДА Донецької області/01/0020/010020-01-00007  <br>3.3G  ./ДА Харківської області/01/0040/0105/010040-105-00969

Explanation:  

On most Linux filesystems (like ext4), a directory with no subdirectories has exactly 2 {{ tag(t="hardlink") }} (`.` and its entry in the parent). This is much faster but may not work on Btrfs or XFS.  
This command is an efficient way to find "leaf" directories (folders containing only files) and sort them by size. Here is the step-by-step breakdown:  
  1) `find . -type d -links 2` This is the "brain" of the command that identifies folders without subfolders.  
  `.` Start searching from the current directory  
  `-type d` Look only for directories.  
  `-links 2` This is a clever trick for most Linux filesystems (like ext4):  
    \- Every directory has a link from its parent and a link to itself . This equals 2 links  
    \- Each subdirectory adds another link to the parent (via ..)  
    \- Therefore, if a directory has exactly 2 links, it means it has zero subdirectories.  
  2) `-exec du -sh {} +` This calculates the size of the folders found.  
  `du -sh` Disk Usage. -s (summary) gives the total size of the folder, and -h (human-readable) converts it to KB, MB, or GB.  
  `{}` This is a placeholder for the list of folders found by find.  
  `+` This tells find to group as many folders as possible into a single du command. This is much faster than running du separately for every single folder.  
  3) `| sort -hr` This organizes the output.  
  `|` The pipe takes the output of the previous command and feeds it into sort.  
  `-h`  Human-numeric sort. It understands that "2G" is larger than "500M". Without this, a standard sort would think "500" is larger than "2" because of the first digit.  
  `-r` Reverse. This puts the largest folders at the top of the list.  

Important Limitation  
This command is extremely fast because it doesn't have to "peek" inside folders to see if they are empty; it just checks the filesystem metadata. However, it may not work on modern filesystems like Btrfs or XFS, as they don't use the standard hard-link counting method for directories.
