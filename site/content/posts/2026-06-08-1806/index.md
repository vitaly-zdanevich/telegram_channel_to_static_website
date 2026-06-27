+++
title = "Wow, about telegram bots: you can bypass 50 MB response limit - by hosting their bot software."
date = 2026-06-08T21:56:16+00:00
description = "Wow, about telegram bots: you can bypass 50 MB response limit - by hosting their bot software. AWS Lambda is not possible here - it wants persistent polling. And it simplify the architecture - less…"

[taxonomies]
tags = ["telegram", "gemini", "lambda"]

[extra]
tg_url = "https://t.me/vitaly_zdanevich_chan/1806"
next_id = 1807
next_title = "love this extension - highlight predefined list of words, on predefined URLs."
prev_id = 1805
prev_title = "My yet another project: @wikipedia_unofficial_bot"
views = 19
ids = [1806]
+++

Wow, about {{ tag(t="telegram") }} bots: you can bypass 50 MB response limit - by hosting their bot software. AWS Lambda is not possible here - it wants persistent polling.

And it simplify the architecture - less nodes in the network chain - when Telegram server and your bot software on the same machine.

{{ tag(t="gemini") }} about why not {{ tag(t="lambda") }}:

> To understand why, you have to look at how Telegram's protocols work beneath the surface.<br><br>When you use the standard public API ([api.telegram.org](http://api.telegram.org/)), your bot makes simple, stateless HTTP requests. However, Telegram's core servers do not speak HTTP; they communicate using a custom, highly encrypted, and stateful protocol called MTProto.<br><br>The public [api.telegram.org](http://api.telegram.org/) server acts as a translator. It maintains persistent MTProto connections to Telegram's core network on your behalf so you do not have to.<br><br>When you run the Local Bot API Server, you are moving that translator to your own machine. Consequently, your local server must now handle the MTProto protocol.<br><br>Here is why that requires a persistent connection and a continuously running server:<br><br>1. Stateful Cryptographic Handshakes: Establishing an MTProto connection is expensive. It requires complex cryptographic handshakes to generate session keys. If you try to run the local server in an ephemeral environment (like spinning it up in AWS Lambda only when you want to send a file), it would have to negotiate a brand new MTProto session from scratch every single time.<br>2. Latency and Rate Limiting: Performing that handshake on every request adds massive latency. More importantly, repeatedly opening and closing MTProto connections to Telegram's core servers will look like a DDoS attack or a malfunctioning client. Telegram's automated systems will likely rate-limit or temporarily ban your bot's IP address for connection spam.<br>3. Local Database State: The Local Bot API Server maintains a local database on the disk to track file IDs, update sequence numbers, and session states. It expects to hold locks on these files and manage them continuously in the background.<br><br>While your bot's code can be executed ephemerally (e.g., waking up only when a webhook is received), the Local Bot API Server it communicates with is designed exclusively as a long-running daemon that holds a persistent MTProto TCP connection to Telegram.

<https://github.com/tdlib/telegram-bot-api>
