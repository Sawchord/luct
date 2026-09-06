+++
title = "luCT"
description = "luCT (pronounced \"lucid\") is a digital self defense tool that adds an extra layer of security to HTTPs by auditing certificate's log inclusion proofs locally in real time, as you browse."
template = "index.html"
+++

## Try it for yourself!

1. [Download](https://download.luct.dev/luct.xpi) the Firefox extension
2. Browse normally
3. Look for the <img src="./icons/luct_safe.svg" alt="luCT icon" width="12em"> indicator when CT inclusion is verified 

<div>
    <img class="example" src="./gifs/example-usage.gif" alt="Example usage of luCT" />
</div>

## Project status 

This project is 🚧 **experimental and not yet audited** 🚧.

- ❗ Bugs are likely
- ❗ False positives/negatives may occur
- ❗ Do not rely on this for critical security decisions

At this point in time, use it for **testing, research, and exploration only**.

## How it works



[Certificate transparency](https://en.wikipedia.org/wiki/Certificate_Transparency) improves web security by requiring certificate authorities to log their certificates in an append only log. Logs return a signed certificate timestamp (SCT) to the certificate authority.

Browsers require SCTs in a certificate when establishing a TLS connection and refuse connection otherwise. However, the SCT is just a signed promise that the certificate will be included in the log eventually. Browsers do not actually communicate with logs to check inclusion proofs of SCTs they find.

luCT closes that gap by checking log inclusions while you browse. If everything checks out, it gives an additional checkmark indicator. In certificate transparency language, this makes luCT an "auditing service", but one that you don't have to trust since it runs entirely in your browser.

## Features and roadmap

### Current features

- ✅ Validate signed certificate timestamps of certificagtes
- ✅ Fetch and update signed tree heads using extension proofs
- ✅ static-ct-api support
- ✅ Oblivious TLS proxy to preserve privacy
- ✅ UI sidebar

### Planned features

- 🚧 STH checkpointing (Gossip)
- 🚧 Stability, performance and UI improvements
- 🚧 UI dashboards for statistics deep dives
- 🚧 Timelocks for Root Certificate authorities
- 🚧 DNS over HTTPs
- 🚧 CAA and TLSA cross-checking

## Sponsors

This project is supported by a grant from [NLNet](https://nlnet.nl/project/luCT/).

[<img src="https://nlnet.nl/logo/banner.svg" atl="luCT icon" height="60">](https://nlnet.nl)
[<img src="https://nlnet.nl/image/logos/NGI0CommonsFund_tag.svg" atl="luCT icon" height="60">](https://nlnet.nl/core/)