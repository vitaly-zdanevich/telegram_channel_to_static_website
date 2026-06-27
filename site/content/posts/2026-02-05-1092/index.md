+++
title = "recording my screen (like previous message) with ffmpeg"
date = 2026-02-05T11:53:36+00:00
description = "recording my screen (like previous message) with ffmpeg: ffmpeg -vaapidevice /dev/dri/renderD128 -f x11grab -videosize 1366x768 -i :0 -vf setpts=N/FR/TB -c:v h264vaapi -vf 'format=nv12,hwupload'…"

[taxonomies]
tags = ["recording", "ffmpeg"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1092"
next_id = 1093
next_title = "My account is big, my account is very big"
prev_id = 1091
prev_title = "linux game wwii landing ad warthunder"
views = 13
ids = [1092]
+++

{{ tag(t="recording") }} my screen (like previous message) with {{ tag(t="ffmpeg") }}:

```
ffmpeg -vaapi_device /dev/dri/renderD128 \
  -f x11grab -video_size 1366x768 \
  -i :0 \
  -vf setpts=N/FR/TB \
  -c:v h264_vaapi -vf 'format=nv12,hwupload' \
  ~/record/out/$(date +%Y-%b-%d%a--%H-%M-%S | tr A-Z a-z).mp4

# https://trac.ffmpeg.org/wiki/Encode/H.264
#
# setpts=N/FR/TB
# to be able to pause by Ctrl-Z, see https://stackoverflow.com/a/61692055/1879101
```
