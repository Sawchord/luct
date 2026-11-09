# Monitors and auditors

In the previous chapters,
we talked about how certificate authorities need to submit certificates to certificate transparency logs.
But what can we actually do with the data in the logs?

The [Standard](todo) mentions two types of actors on the log data: auditors and monitors

## Monitors

Monitors are services that scan and validate through entire logs.
They build database indexes of the data contained within the logs.

The most straightforward use case of a monitor is
getting an overview of all existing certificates for a website,
and possibly spot rogue certificates.
Of course, the last part can only be done by a website administrator,
because only they have can differentiate between a legitimate and an illegitimate certificate.

Ideally, the monitors also validate the inclusion proofs of the contained certificate,
but unfortunately not all monitors do this.

A famous, free to use monitor is [crt.sh](crt.sh).
Give it a try, if you haven't already!

There are a number of monitoring services in existence.
An overview can be found [here](todo)

## Auditors

Auditors are another important puzzle piece.
They provide the link between the signed certificate timestamps and the log entries.

SCTs are only signed promised, that the certificate will be included in the log.
The browsers only check that SCTs are present and valid in a certificate,
they do not actually check that the SCT has a matching log entry with a valid inclusion proof.

There are some good reasons for that:

- Fetching log data increases the data usage
- It can leak data about which websites you use to the log operators
- There might be intermittent issues such as a log being unavailable, that need to be handled

For these reasons, it makes sense that browsers do not actually audit the SCTs by default.
However you should still be **able** to audit SCTs, if you suspect that rogue certificates are a threat.

The standard document envisioned auditing as a service similar to monitoring,
where you sent your SCTs to and get back audit proofs.
However, this type of auditing service has never really materialized
(at least not as a public service).

Interestingly, auditors require only a small amount of the log data to work correctly.
It is totally feasible to do auditing right in the browser,
with only very lightweight additional infrastructure required,
that you don't need to trust for integrity.

This is exactly what luCT is.
It is an auditor that runs as a browser extension in the background
and audits the SCTs from certificates that your browser sees when it visits website.
