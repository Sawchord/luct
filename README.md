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
It is neither feature complete, nor has it been independently audited. Both false positive and false negative results due to bugs are still possible and likely at this point in time.

**Use at your own risk! Do not critically rely on output from this tool yet!**

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