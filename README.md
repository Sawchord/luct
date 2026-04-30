# luCT

luCT (pronounced "lucid") is a digital self defense tool to audit certificate transparency logs in the browser.

## Quick Start

1. Install the Firefox extension
2. Browse normally
3. Look for the indicator when CT inclusion is verified


## Status

This project is **experimental and not yet audited**.

- Bugs are likely
- False positives/negatives may occur
- Do not rely on this for critical security decisions

At this point in time, use it for **testing, research, and exploration only**.


## Building

To build luCT yourself, see [build instructions](./BUILD.md)


## How this works

[Certificate transparency](https://en.wikipedia.org/wiki/Certificate_Transparency) helps to improve the security of the Web by requiring certificate authorities to log their certificates in an append only log.
Logs return a signed certificate timestamp (SCT) to the certificate authority.

Browsers require that there are SCTs in a certificate when establishing a TLS connection and refuse connection otherwise.
However, the SCT is just a signed promise that the certificate will be included in the log eventually.
Browsers do not actually follow the logs and check inclusion proofs of SCTs that they come by.

luCT closes that gap by checking log inclusions while you browse.
If everything checks out, it gives an additional checkmark indicator.
It is planned to extend luCT over time, such that it includes more and more guarantees over time. 

## Privacy

Querying CT logs directly reveals which sites you visit.

luCT avoids this using an **oblivious TLS proxy**:

- Proxy sees your IP, but not your request
- Log sees the request, but not your IP

**Result**: no single party can link you to your browsing activity

If you use a VPN or Tor, you can disable the proxy.

luCT does **not collect any telemetry**.


## Permissions

TODO


## Repository overview

Here is a short overview of what is where in the repository

- `extension/src`: Firefox extension svelte UI
- `extension/luct`: Firefox extension data, manifest, etc.
- `luct`: Luct CLI tool
- `luct-client`: Implementation of clients to connect to logs and fetch data.
- `luct-core`: CT parsing and verification (RFC6962)
- `luct-extension`: Rust side of the firefox extension.
    Implements a wrapper around `luct-scanner` and some necessary infrastructure for running it in a browser.
- `luct-node`: Executable that supports server functionality used in `luct-scanner`, such as the oblivious TLS proxy.
    This is NOT a log implementation.
- `luct-otlsp`: Integration of `otlsp-client` as a `luct-client`
- `luct-scanner`: Core auditing logic.
- `luct-server`: Placeholder for potential future log implementation
- `luct-store`: Implementations to store CT artifacts.
- `luct-test`: Collection of test code used in development
- `otlsp-*core*`: Oblivious TLS proxy implementation


## License

At your discretion:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)