# luCT

luCT (pronounced "lucid") is a tool to audit Certificate Transparentcy logs in the browser.


## Security Warning

The project is a very early work-in-progress.
It is neither feature complete, nor has it been independently audited. Use at your own risk!
**Do not rely on output from this tool!**

## Privacy warning

Requesting an audit proof from the log will leak the information, which certificates you are interested in (and therefore, which websites you visit) to the log provider.
This is likely the main reason why browsers do not check the audit proofs by default.

There are multiple avenues to remedy this situation, but none are implemented yet.
For now, the only privacy preserving mode of operating this tool is to use it behind a VPN or through TOR.

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