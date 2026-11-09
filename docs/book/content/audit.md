# What is luCT

Now that we know how certificate transparency works,
we might ask ourselves once more:

Is this all secure now?

And again, the answer is that by requiring certificates to be logged,
we increased the security **by a huge lot**.
Silently issuing a rogue certificate is way harder than it used to be.
However, there are still ways, in which a TLS connection can be attacked.
For more details, see the chapter of the luCT [thread model](security/threat_model.md).

One might argue, that what we have now it is secure enough for most applications.
However, I would argue that is not secure enough for **all** applications.

There might be professions or people that handle especially sensitive data or
are particularly vulnerable to surveillance and and these people need to use the internet too.

The idea of luCT is to try to close as many of these gaps as possible.
This manual is supposed to help you understand what luCT does, and how this helps your privacy.

In this chapter, we will go through how luCT ties into certificate transparency and how to use it.
[The next chapter](security.md) will explain in detail, how luCT operates,
what is checked and what security that provides.

It should be clear from the get go, that luCT is not a piece of magic,
that makes all possible attacks on your privacy go away.
To browse the internet as securely as possible,
there are also other steps that need to be taken in conjunction with using luCT
(see the [security beyond luCT](security/outlook.md) chapter for more information).
