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
og_image = "https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/releases/download/images-1000/telegram-image-1106-5215513357908121079.jpg"
next_id = 1107
next_title = ""
next_body = "Author: Елена Запрудская"
prev_id = 1105
prev_title = ""
prev_body = "#commons\nThe server did not respond within the expected time\nIf you cannot upload your big #pdf - you can extract all images from it, with original quality:\nimport fitz # PyMuPDF\ndoc = fitz.open('yourfile.pdf')\nfor pageindex in range(len(doc)):\nfor imgindex, img in enumerate(doc.getpageimages(pageindex)):\nxref = img[0]\nbaseimage = doc.extractimage(xref)\nimagebytes = baseimage['image']\nimageext = baseimage['ext'] # Preserve original format (e.g., 'jpeg', 'png', 'jp2')\nwith open(f'page{pageindex+1}{imgindex+1}.{imageext}', 'wb') as f:\nf.write(imagebytes)\nand upload through my #pywikibot wrapper"
views = 19
ids = [1106]

[[extra.related]]
path = "@/posts/2026-02-02-1073/index.md"
label = "#wikipedia Актёр озвучивания мужского пола Монгильо наиболее изв…"
date = "2026-02-02"

[[extra.related]]
path = "@/posts/2025-04-25-483/index.md"
label = "My new article on #wikipedia"
date = "2025-04-25"

[[extra.related]]
path = "@/posts/2024-11-25-201/index.md"
label = "#wikipedia"
date = "2024-11-25"

[[extra.related]]
path = "@/posts/2026-03-25-1503/index.md"
label = "#wikipedia #wikimediacommons Пишите авторам контентов - иногда о…"
date = "2026-03-25"

[[extra.related]]
path = "@/posts/2026-02-14-1112/index.md"
label = "Editing #wikipedia, in #vim"
date = "2026-02-14"
+++

{{ tag(t="wikipedia") }}  
{{ tag(t="job") }}

{{ img(src="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/releases/download/images-1000/telegram-image-1106-5215513357908121079.jpg") }}
