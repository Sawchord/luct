# Certificate authorities

The first party that comes into play are the Certificate Authorities (CAs).

## Transport layer security

When you are browsing the web, your connection is (hopefully) secured by a protocol
call transport layer security (TLS).
You can see that by looking at your url, which hopefully will start with `https://`.
In this day and age, unsecured connectsion (`http://`) are largely a thing of the past.

TLS will establish a secure channel between you and the server you are connecting to.
How this works in general is not of interest right now.
However, there is piece of this TLS connection that is interesting:
While TLS allows you to securely connect to a server, how do you know that it is **THE** server.
An attacker might just intercept your connection, and then make it's own connection
to the original server.

In other words, when you visit `https://luct.dev`, you don't just want to talk TLS to
any server, but to the server that serves `luct.dev` specifically.
The server will prove this to you via a cryptographic proof that links it public key
to the TLS connection you just established.
If you know, which public key goes with which URL, this proves that you are actually
talking to the correct counterparty.

But this creates a new problem:

**How do you know which public key goes with which server?**

For this to work, you would have to have a map from public key to url stored on you computer.
This is not realistic, it would be like a phonebook for the entire internet.
It also would need to be updated all the time.

## Certificates

This is were certificates enter the stage.
Instead of having a phonebook of the entire internet, the server will present to you a
document, called a **certificate**, that proves that it's key is valid for the website you
are just visitng.

Now, how do you know, that the server didn't just make up this document by itself?
The document is signed by a **Certificate Authority** (CA), that attests it's correctness.
This sounds like we just shifted the problem of having a phonebook of the internet to
having a phonebook of all certificate authorities, and in fact, we did.
But there are much less certificate authorities to keep track of, than there are urls.

In reality, there are still too many vertificate authorities too keep track of, 
but we can play this game multiple times.
Instead of presenting a single certificate, the server presents a **certificate chain**,
from it's own certificate up to a **root certificate**.
Usually, this chain is three to four certificates long.

The root certificate is issued by a **root certificate authority** (Root CA).
And yes, your browser has in fact a phonebook with all root CAs that it trusts built in.
This phonebook is called the **trust store**.

If you are curious what organizations are in there, take a look 
[here](https://ccadb.my.salesforce-sites.com/mozilla/IncludedRootCertificateReport).