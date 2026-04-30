# luCT

luCT (pronounced "lucid") is a digital self defense tool to audit certificate transparency logs in the browser.


## Caution!

The project is a work-in-progress.
It is neither feature complete, nor has it been independently audited.
Both false positive and false negative results due to bugs are still possible and likely at this point in time.

**Use at your own risk! Do not critically rely on output from this tool yet!**


## How this works

[Certificate transparency](https://en.wikipedia.org/wiki/Certificate_Transparency) is a standard and an ecosystem to improve the security of the Web, by requiring certificate authorities to log each certificate they issue in an append only log.
Upon logging the certificate, logs return a signed certificate timestamp (SCT) to the certificate authority.

Browsers require that there are SCTs embedded into the certificate when establishing a TLS connection and refuse connection otherwise.
However, the SCT is just a signed promise that the certificate will be included in the log eventually.
Browsers do not actually follow the logs and check inclusion proofs of SCTs that they come by. 
This is where luCT comes in!

luCT is a Firefox extension that checks log inclusions while you browse.
If everything checks out, it gives an additional checkmark indicator.
It is planned to extend luCT over time, such that it includes more and more guarantees over time. 


## Privacy

Requesting an audit proof from the log will leak the information, which certificates you are interested in (and therefore, which websites you visit) to the log provider.

To prevent this, luCT uses an "oblivious TLS proxy".
luCT does not connect to the logs directy, but instead wraps the connection into a seconds TLS connection that is sends via the proxy.

The proxy sees your IP address, but does not learn anything about the request.
The log sees which certificates you are interested in but not your IP address.

If you use luCT through a VPN or TOR, you can deactivate the oblivious TLS proxy in the settings.

luCT does not collect any metrics or other information about the user.


## Permissions

## Repository overview

Here is a short overview of what is where in the repository

- `extension`: Directory for the firefox extension JavaScript.
    Contains two subdirectories `src`, which is the UI svelte code and `luct`, which contains extension files
- `luct`: Luct CLI tool
- `luct-client`: Implementation of clients to connect to logs and fetch data.
- `luct-core`: Core RFC6962 implementation. 
    Contains the code for parsing and verifying CT artifacts.
- `luct-extension`: Rust side of the firefox extension.
    Implements a wrapper around `luct-scanner` and some necessary infrastructure for running it in a browser.
- `luct-node`: Executable that supports server functionality used in `luct-scanner`, such as the oblivious TLS proxy.
    This is NOT a log implementation.
- `luct-otlsp`: Integration of `otlsp-client` as a `luct-client`
- `luct-scanner`: Implements the core auditing logic.
- `luct-server`: Placeholder crate for a potential future log implementation
- `luct-store`: Different implementations to store CT artifacts.
    Mainly used by `luct-scanner`
- `luct-test`: Collection of test code used in development
- `otlsp-core`: Shared code for oblivious TLS proxy implementation
- `otlsp-client`: Client side of oblivious TLS proxy implementation
- `otlsp-server`: Server side of oblivious TLS proxy implementation


## License

At your discretion:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)