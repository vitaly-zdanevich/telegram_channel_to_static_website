+++
title = ""
date = 2026-01-21T23:09:03+00:00
description = "Did you know about git notes? Adds, removes, or reads notes attached to objects, without touching the objects themselves. By default, notes are saved to and read from refs/notes/commits, but this…"

[taxonomies]
days = ["2026-01-21"]

[extra]
id = 930
day = "2026-01-21"
tg_url = "https://t.me/vitaly_zdanevich_chan/930"
next_id = 931
next_title = ""
next_body = "I am on #gentoo because it compiles for my CPU (-march=native and other flags - not generic x64, but I did not measure the performance numbers), and another point - USE flags and the ability to apply source based patches. For example in Firefox in about:firefoxview in Open tabs we have only 7 elements and no preferences to increase that number, so I found it in the source code and produced a simple patch for my system - and on every update the Portage package manager will try to apply that patch."
prev_id = 929
prev_title = ""
prev_body = "#belarus\n#church\n#year2005\n#globustut"
views = 13
ids = [930]
+++

Did you know about [git notes](https://git-scm.com/docs/git-notes)?  

> Adds, removes, or reads notes attached to objects, without touching the objects themselves.  <br>  <br>By default, notes are saved to and read from refs/notes/commits, but this default can be overridden. See the OPTIONS, CONFIGURATION, and ENVIRONMENT sections below. If this ref does not exist, it will be quietly created when it is first needed to store a note.  <br>  <br>A typical use of notes is to supplement a commit message without changing the commit itself. Notes can be shown by git log along with the original commit message. To distinguish these notes from the message stored in the commit object, the notes are indented like the message, after an unindented line saying "Notes (&lt;refname&gt;):" (or "Notes:" for refs/notes/commits).  <br>  <br>Notes can also be added to patches prepared with git format-patch by using the --notes option. Such notes are added as a patch commentary after a three dash separator line.
