# Introduction

Welcome to the luCT manual.

The purpose of this document is to help you understand what luCT does,
in which ways using luCT makes your browsing more secure,
and crucially, in which ways it **does not**.

luCT is built on a stack of pre-existing technologies, namely
- Tranport layer security (TLS)
- Certificate authorities (CAs)
- Certificate transparency logs (CTs)

[In the first chapter](primer.md), we will give a brief introduction into
what these technologies are, who runs them, and how they interact.

[In the second chapter](audit.md), we will show how luCT ties into this, and
what it can do on top these existing technologies.

[The third chapter](security.md) will go in more detail into the design of luCT
and it's threat model and features.

> [!CAUTION]
> luCT is a security tool and if you want to use it as such,
> you need to understand what it can and can not do for you.
>
> Furthermore **luCT is still in alpha** and
> is being actively worked on.
> 
> So for the time being, keep in mind:
> - ❗ Bugs are likely
> - ❗ False positives/negatives may occur
> - ❗ Do not rely on this for critical security decisions
>

That said, **you should definitely try out luCT!**.

The tool is very lightweight and it can give you interesting
insights into the security architecture that you rely on every day
to browse the internet securely.

Just keep in mind that at this point in time, luCT should be used as
an exploration and research tool, not as a security tool.

It will hopefully be a security tool one day, but we are just not
there yet.