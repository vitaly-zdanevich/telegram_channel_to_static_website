+++
title = ""
date = 2026-02-10T19:31:40+00:00
description = "cd to a first sorted folder, exclude hidden first() { local first first=\"$( find . -mindepth 1 -maxdepth 1 -type d ! -name '.' -printf '%fn' | sort | head -n 1 )\" if [[ -z \"$first\" ]]; then echo \"no…"

[taxonomies]
days = ["2026-02-10"]

[extra]
id = 1104
day = "2026-02-10"
tg_url = "https://t.me/vitaly_zdanevich_chan/1104"
next_id = 1105
next_title = ""
next_body = "#commons\nThe server did not respond within the expected time\nIf you cannot upload your big #pdf - you can extract all images from it, with original quality:\nimport fitz # PyMuPDF\ndoc = fitz.open('yourfile.pdf')\nfor pageindex in range(len(doc)):\nfor imgindex, img in enumerate(doc.getpageimages(pageindex)):\nxref = img[0]\nbaseimage = doc.extractimage(xref)\nimagebytes = baseimage['image']\nimageext = baseimage['ext'] # Preserve original format (e.g., 'jpeg', 'png', 'jp2')\nwith open(f'page{pageindex+1}{imgindex+1}.{imageext}', 'wb') as f:\nf.write(imagebytes)\nand upload through my #pywikibot wrapper"
prev_id = 1103
prev_title = ""
prev_body = "My new #bash alias: one function to go to the next folder (like from 2025 to 2026, from aaa to bbb) and the second one to #cd to prev:\ncdn() {\nlocal cur next\ncur=\"$(basename \"$PWD\")\"\nnext=\"$(\nfind .. -mindepth 1 -maxdepth 1 -type d -printf '%fn'\n| sort\n| awk -v cur=\"$cur\" '$1cur{print; exit}'\n)\"\nif [[ -z \"$next\" ]]; then\necho \"no next folder\"\nreturn 1\nfi\ncd \"../$next\"\n}\ncdp() {\ncur=\"$(basename \"$PWD\")\"\nprev=\"$(\nfind .. -mindepth 1 -maxdepth 1 -type d -printf '%fn'\n| sort\n| awk -v cur=\"$cur\" '$1<cur{p=$1} END{print p}'\n)\"\nif [[ -z \"$prev\" ]]; then\necho \"no previous folder\"\nreturn 1\nfi\ncd \"../$prev\"\n}"
views = 23
ids = [1104]
+++

```
# cd to a first sorted folder, exclude hidden
first() {
  local first
  first="$(
    find . -mindepth 1 -maxdepth 1 -type d ! -name '.*' -printf '%f\n' \
    | sort \
    | head -n 1
  )"

  if [[ -z "$first" ]]; then
    echo "no folders"
    return 1
  fi

  cd "./$first"
}
```
