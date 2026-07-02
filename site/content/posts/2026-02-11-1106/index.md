+++
title = ""
date = 2026-02-11T10:47:08+00:00
description = "wikipedia job"

[taxonomies]
days = ["2026-02-11"]
tags = ["wikipedia", "job"]

[extra]
id = 1106
day = "2026-02-11"
tg_url = "https://t.me/vitaly_zdanevich_chan/1106"
og_image = "5215513357908121079_1214331332_460002807.jpg"
next_id = 1107
next_title = ""
next_body = "Author: Елена Запрудская"
prev_id = 1105
prev_title = ""
prev_body = "#commons\nThe server did not respond within the expected time\nIf you cannot upload your big #pdf - you can extract all images from it, with original quality:\nimport fitz # PyMuPDF\ndoc = fitz.open('yourfile.pdf')\nfor pageindex in range(len(doc)):\nfor imgindex, img in enumerate(doc.getpageimages(pageindex)):\nxref = img[0]\nbaseimage = doc.extractimage(xref)\nimagebytes = baseimage['image']\nimageext = baseimage['ext'] # Preserve original format (e.g., 'jpeg', 'png', 'jp2')\nwith open(f'page{pageindex+1}{imgindex+1}.{imageext}', 'wb') as f:\nf.write(imagebytes)\nand upload through my #pywikibot wrapper"
views = 19
ids = [1106]
+++

{{ tag(t="wikipedia") }}  
{{ tag(t="job") }}

![](5215513357908121079_1214331332_460002807.jpg)
