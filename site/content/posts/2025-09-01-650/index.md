+++
title = ""
date = 2025-09-01T04:23:08+00:00
description = "Wow in svg we can have css, like .spinnera { animation: spinnerMGfb .8s linear infinite; animation-delay: -.8s; } .spinnerb { animation-delay: -.65s; } .spinnerc { animation-delay: -.5s; } @keyframes…"

[taxonomies]
days = ["2025-09-01"]
tags = ["svg", "css"]

[extra]
id = 650
day = "2025-09-01"
tg_url = "https://t.me/vitaly_zdanevich_chan/650"
next_id = 651
next_title = ""
prev_id = 649
prev_title = ""
views = 39
ids = [650]
+++

Wow in {{ tag(t="svg") }} we can have {{ tag(t="css") }}, like

```
<svg width='24' height='24' viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>

  <style>
    .spinner_a {
      animation: spinner_MGfb .8s linear infinite;
      animation-delay: -.8s;
    }

    .spinner_b {
      animation-delay: -.65s;
    }

    .spinner_c {
      animation-delay: -.5s;
    }

    @keyframes spinner_MGfb {
      0%   { fill: black; }
      50%  { fill: white; }
      100% { fill: black; }
    }
  </style>

  <circle class='spinner_a' cx='4' cy='12' r='3'/>
  <circle class='spinner_a spinner_b' cx='12' cy='12' r='3'/>
  <circle class='spinner_a spinner_c' cx='20' cy='12' r='3'/>

</svg>
```

and it works with usual `<img src='my.svg'/>`
