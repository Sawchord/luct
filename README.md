# luCT

luCT (pronounced "lucid") is a set of tools to increase trust into the WebPKI ecosystem.

It's goal is to provide tools to interact with Certificate Transparency (CT) logs.

## Security Warning

The project is a very early work-in-progress.
It is neither feature complete, nor has it been independently audited. Use at your own risk!
**Do not rely on output from this tool!**

## Scanner

> Status: **WIP**

The scanner is a tool that is configured with a number of logs.
It is able to follow the Signed Tree Heads (STHs) of the logs and
validate the consistency proofs.

The scanner can scan a certificate chain, verify it, extract the
Signed Certificate Timestamps (SCTs) and query the logs for audit
proofs, which it verifies.

Additionally, the scanner can enforce simple rules, such as blocking
certain root or intermediate CAs.

The scanner tool is exposed as a cli in the `luct` crate.

### SCT sources

At the moment, the scanner only verifies embedded SCTs, which it
extracted from the certificate itself.
In the future, the scanner could also get additional certificates from via OCSP or TLS stapling.

### Privacy

Requesting an audit proof from the log will leak the information,
which certificates you are interested in (and therefore, which websites
you visit) to the log provider.
This is likely the main reason why browsers do not check the audit
proofs by default.

If you are behind a VPN or inside the TOR network, this information is
hidden. 


## Extension

> Status: **TODO**

A Firefox extension that contains the scanner compiled to webassembly.
It runs the scanner on all URLs that the user visits, and indicates
a succesfull check through a small icon in the task bar.

## Server

> Status: **TODO**

An implementation of a RFC 6962 server.
The server can be configured to run multiple instances at once.
The instances can be configured to be
- `primary`: active logs which accept certificates for loging and hand sout SCTs
- `monitors`: follows another log, checks it's audit proofs and logs certificates it has seen for analysis
- `mirror`: Like monitor, but additionally keeps the merkle tree around and can thus serve RFC 6962 requests just like a `primary`

## Crates

Here is a short list of crates contained in this repository,
including a short decription of their purpose:

- `luct-core`: Core RFC6962 implementation. Contains the code for
parsing and verifying CT artifacts.
- `luct-client`: Async client implemtation. Uses `luct-core` and
implements a stateless client over it.
The underlying HTTP client is pluggable.
- `luct-scanner`: Implementation of the scanner
- `luct-server`: Implementation of the server / monitor
- `luct-store`: Multiple `Store` implementations, which are used
by `luct-scanner` and `luct-server`.
- `luct`: CLI tool to run `luct-scanner` in the console
- `luct-extension`: Firefox web-extension to run `luct-scanner` in firefox

## License

At your discretion:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)