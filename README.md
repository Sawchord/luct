# luCT

luCT (pronounced "lucid") is a tool to audit certificate transparentcy logs in the browser.


[Certificate transparency](https://en.wikipedia.org/wiki/Certificate_Transparency) is a standard and an ecosystem to improve the security of the Web, by requiring certificate authorities to log each certificate they issue in an append only log.
Upon logging the certificate, logs return a signed certificate timestamp (SCT) to the certificate authority.

Browsers require that there are SCTs embedded into the certificate when establishing a TLS connection and refuse connection otherwise.
However, the SCT is just a signed promise that the certificate will be included in the log eventually.
Browsers do not actually follow the logs and check inclusion proofs of SCTs that they come by. 
This is where luCT comes in!

It is a Firefox extension that checks log inclusions while you browse.
If everything checks out, it gives an additional checkmark indicator.
It is planned to extend luCT over time, such that it includes more and more guarantees over time. 

## Security Warning

The project is a work-in-progress.
It is neither feature complete, nor has it been independently audited.
Both false positive and false negative results due to bugs are still possible and likely at this point in time.

**Use at your own risk! Do not critically rely on output from this tool yet!**

## Privacy warning

Requesting an audit proof from the log will leak the information, which certificates you are interested in (and therefore, which websites you visit) to the log provider.
This is likely the main reason why browsers do not check the audit proofs by default.

There are multiple avenues to remedy this situation, but none are implemented yet.
For now, the only privacy preserving mode of operating this tool is to use it behind a VPN or through TOR.

## Repository

Here is a short overview of what is where in the repository

- `extension`:

    Directory for the firefox extension JavaScript.
    Contains two subdirectories `src`, which is the UI svelte code and `luct`, which contains extension files
    
- `luct`: Luct CLI tool
- `luct-client`:

    Implementation of clients to connect to logs and fetch data.

- `luct-core`: 

    Core RFC6962 implementation. 
    Contains the code for parsing and verifying CT artifacts.

- `luct-extension`:
    
    Rust side of the firefox extension.
    Implements a wrapper around `luct-scanner` and some necessary infrastructure for running it in a browser.

- `luct-node`:

    Executable that supports server functionality used in `luct-scanner`, such as the oblivious TLS proxy.
    This is NOT a log implementation.

- `luct-otlsp`: Integration of `otlsp-client` as a `luct-client`
- `luct-scanner`: Implements the core auditing logic.

- `luct-server`: Placeholder crate for a potential future log implementation
- `luct-store`: 

    Different implementations to store CT artifacts.
    Mainly used by `luct-scanner`

- `luct-test`: Collection of test code used in development
- `otlsp-core`: Shared code for oblivious TLS proxy implementation
- `otlsp-client`: Client side of oblivious TLS proxy implementation
- `otlsp-server`: Server side of oblivious TLS proxy implementation


## License

At your discretion:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)