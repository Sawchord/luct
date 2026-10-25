# Certificate transparency logs

## Why do we need certificate transparency

Now, with the certificate authorities in place, everything is secure right?

Well, it is in fact **much more** secure than it was before.
However, if you look into the [list](https://ccadb.my.salesforce-sites.com/mozilla/IncludedRootCertificateReport)
of root certificate authorities, you will that it is a wild mix of organisations from all over the world.

We have:
- commercial entities such as **Google** and **Amazon** running a for profit business
- non-profit organisations such as the **Internet Security Research Group (Let's Encrypt)**
- research organisations, such as **Hellenic Academic and Research Institutions**
- state institutions, like the **Government of Spain** and the **Goverment of the Netherlands**

With some restrictions, that I will glance over here, any root CA can issue certificates for any website.
So, when you trust a certificate chain, you are not only trusting every entity in the chain to be
honest and capable of maintaining security, you are in fact trusting **all** of the organizations
in the list.

If any organisation in this list issues a certificate for a website the owner of that website hasn't 
asked for it is called a **rogue certificate**.

The reason for this happening are manifold:
- they could be tricked by someone assuming someone else's identity
- they could have bad security and a hacker steals the secret key and issues their own certificates
- they (or someone within the org) could be corrupt and hand out rogue certificates for money
- they could act on behalf of some government in an attempt to surveil their own or another countries citizen

Now, most certificate authorities are doing a good job, and there is an auditing system in place.
However, keep in mind that we are relying on **all of them** to work correctly.
Given that these are organisations with vastly different internal compositions and reasons-d'etre
which are also spread all around the globe, it is a big ask to trust them all.

In fact, incidents are happening at a low but steady pace.
One of the most prominent of these cases is the case of [DigiNotar](todo).

The crux of the problem is that issuing a certificate is something that CAs do by themselves
and no one else is in the loop.
If there was some mechanism to ensure that all certificates have to be publicly visible,
we could built mechanisms to check for weird and unexpected certificates,
revoke them and discover broken CAs.

This is what the **certificate transparency** (CA) system does!.

## How certificate transparency works

