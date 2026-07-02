+++
title = ""
date = 2026-06-14T16:41:55+00:00
description = "vim java hover balloon with type definition ftplugin/java.vim, with coc highlight CocTypePopup ctermfg=White ctermbg=22 guifg=ffffff guibg=005f00 function! CocTypeBalloonExpr() abort try let l:docs =…"

[taxonomies]
days = ["2026-06-14"]
tags = ["vim", "java", "hover", "balloon", "coc"]

[extra]
id = 1825
day = "2026-06-14"
tg_url = "https://t.me/vitaly_zdanevich_chan/1825"
next_id = 1826
next_title = ""
next_body = "#bus\n#architecture\nSource"
prev_id = 1824
prev_title = ""
prev_body = "#wikipedia\n#armenian\n#ruwiki\n#language\n#table"
views = 19
ids = [1825]
+++

{{ tag(t="vim") }}  
{{ tag(t="java") }}  
{{ tag(t="hover") }} {{ tag(t="balloon") }} with type definition `ftplugin/java.vim`, with {{ tag(t="coc") }}  

```
highlight CocTypePopup ctermfg=White ctermbg=22 guifg=#ffffff guibg=#005f00

function! CocTypeBalloonExpr() abort

  try
    let l:docs = CocAction('getHover', {
      \ 'bufnr': v:beval_bufnr,
      \ 'line': v:beval_lnum,
      \ 'col': v:beval_col
      \ })
  catch
    return ''
  endtry

  for l:line in split(join(l:docs, "\n"), "\n")
    let l:line = trim(l:line)
    call popup_beval(l:line, {
      \ 'maxwidth': 200,
      \ 'padding': [0, 1, 0, 1],
      \ 'border': [0, 0, 0, 0],
      \ 'highlight': 'CocTypePopup',
      \ 'wrap': 1
      \ })
    return ''
  endfor

  return ''
endfunction

if !has('nvim')
  setl balloonevalterm
  setl balloonexpr=CocTypeBalloonExpr()
endif
```
