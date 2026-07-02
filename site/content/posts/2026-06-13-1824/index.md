+++
title = ""
date = 2026-06-13T16:39:45+00:00
description = "wikipedia armenian ruwiki language table"

[taxonomies]
days = ["2026-06-13"]
tags = ["wikipedia", "armenian", "ruwiki", "language", "table"]

[extra]
id = 1824
day = "2026-06-13"
tg_url = "https://t.me/vitaly_zdanevich_chan/1824"
og_image = "5289874428806242056_1231644868_460005128.jpg"
next_id = 1825
next_title = ""
next_body = "#vim\n#java\n#hover #balloon with type definition ftplugin/java.vim, with #coc\nhighlight CocTypePopup ctermfg=White ctermbg=22 guifg=#ffffff guibg=#005f00\nfunction! CocTypeBalloonExpr() abort\ntry\nlet l:docs = CocAction('getHover', {\n'bufnr': v:bevalbufnr,\n'line': v:bevallnum,\n'col': v:bevalcol\n})\ncatch\nreturn ''\nendtry\nfor l:line in split(join(l:docs, \"n\"), \"n\")\nlet l:line = trim(l:line)\ncall popupbeval(l:line, {\n'maxwidth': 200,\n'padding': [0, 1, 0, 1],\n'border': [0, 0, 0, 0],\n'highlight': 'CocTypePopup',\n'wrap': 1\n})\nreturn ''\nendfor\nreturn ''\nendfunction\nif !has('nvim')\nsetl balloonevalterm\nsetl balloonexpr=CocTypeBalloonExpr()\nendif"
prev_id = 1823
prev_title = ""
prev_body = "#shell\n#productivity\n#love my mg alias - clickable #grep in #kitty - opens file and line in Vim:\n# Grep, click to link - open in Vim, exact line\nmg() {\nkitty +kitten hyperlinkedgrep --smart-case -C 9 \"$@\"\n}\n-C 9 is the context - to have a few lines before and after.\nFor this, also you need to have in /.config/kitty/open-actions.conf:\nprotocol file\nfragmentmatches [0-9]+\naction launch --type=overlay -- vim +$FRAGMENT -- $FILEPATH"
views = 18
ids = [1824]
+++

{{ tag(t="wikipedia") }}  
{{ tag(t="armenian") }}  
{{ tag(t="ruwiki") }}  
{{ tag(t="language") }}  
{{ tag(t="table") }}  

[https://ru.wikipedia.org/wiki/Армянский\_язык](https://ru.wikipedia.org/wiki/%D0%90%D1%80%D0%BC%D1%8F%D0%BD%D1%81%D0%BA%D0%B8%D0%B9_%D1%8F%D0%B7%D1%8B%D0%BA)

![](5289874428806242056_1231644868_460005128.jpg)
