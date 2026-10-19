+++
title = "luCT"
description = "luCT (pronounced \"lucid\") is a Firefox extension that checks whether website certificates have been recorded in public certificate logs. It adds an extra layer of security to HTTPS by verifying inclusion proofs locally as you browse."
template = "index.html"
+++

## Try it for yourself!

Requires **Firefox 144 or newer**. The download is an extension file (`.xpi`) hosted by the project.

1. Open the [luCT download](https://download.luct.dev/luct.xpi) in Firefox and follow the installation prompts.
2. Browse normally
3. Look for the <img src="./icons/luct_safe.svg" alt="luCT icon" width="12em"> indicator when CT inclusion is verified

If Firefox saves the file instead, open **Add-ons and themes**, select **Extensions**, then use the gear menu to choose **Install Add-on From File…** and select the downloaded `.xpi` file. Review the requested permissions and confirm.

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



[Certificate Transparency (CT)](https://en.wikipedia.org/wiki/Certificate_Transparency) improves web security by requiring certificate authorities to log publicly trusted certificates in append-only logs. Logs return a signed certificate timestamp (SCT) to the certificate authority.

Browsers require SCTs in a certificate when establishing a TLS connection and refuse connection otherwise. However, the SCT is just a signed promise that the certificate will be included in the log eventually. Browsers do not actually communicate with logs to check inclusion proofs of SCTs they find.

luCT closes that gap by checking log inclusions while you browse. When these checks pass, luCT displays a checkmark for verified log inclusion. In certificate transparency language, this makes luCT an "auditing service", but one that you don't have to trust since it runs entirely in your browser.

## Features and roadmap

### Current features

- ✅ Validate signed certificate timestamps embedded in certificates
- ✅ Fetch and update signed tree heads (STHs) using extension proofs
- ✅ static-ct-api support
- ✅ Oblivious TLS proxy to preserve privacy
- ✅ UI sidebar

### Planned features

- 🚧 STH checkpointing (using a gossip protocol)
- 🚧 Stability, performance and UI improvements
- 🚧 UI dashboards for statistics deep dives
- 🚧 Timelocks for Root Certificate authorities
- 🚧 DNS over HTTPS
- 🚧 CAA and TLSA cross-checking

## Sponsors

This project is supported by a grant from [NLNet](https://nlnet.nl/project/luCT/).

[<img src="https://nlnet.nl/logo/banner.svg" alt="NLnet" height="60">](https://nlnet.nl)
[<img src="https://nlnet.nl/image/logos/NGI0CommonsFund_tag.svg" alt="NGI Zero Commons Fund" height="60">](https://nlnet.nl/core/)