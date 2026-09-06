+++
title = "luCT"
description = "luCT (pronounced \"lucid\") is a digital self defense tool that adds an extra layer of security to HTTPs by auditing certificate's log inclusion proofs locally in real time, as you browse"
template = "index.html"
+++

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

### MVP

- ✅ Validate inclusion proofs of signed certificate timestamps
- ✅ Fetch and update signed tree heads
- ✅ Validate extension proofs
- ✅ static-ct-api support
- ✅ Oblivious TLS proxy to preserve privacy
- ✅ UI sidebar
- ✅ Publish in Mozilla extension store

### Alpha (Planned features)

- 🚧 Stability, performance and UI improvements
- 🚧 UI dashboards for statistics deep dives
- 🚧 STH checkpointing (Gossip)
- 🚧 Oblivious TLS proxy for CLI
- 🚧 Timelocks for Root Certificate authorities

### Post release (Ideas for future extensions)

- 🚧 DNS over HTTPs
- 🚧 CAA and TLSA cross-checking