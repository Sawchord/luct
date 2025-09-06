# End to end test

## Preliminaries

This directory consists of an end to end test, testing both the server and the backend.
It assumes, that `miniserve` is installed on the system.
It also uses the `wasm-pack`, which can be installed using `cargo` as well.

To install `miniserve`, run:

```bash
cargo install -f miniserve wasm-pack
```

The certificates where generated using `openssl` and the following commands
(assuming `openssl` is installed on the systed):
```bash
openssl ecparam -name secp384r1 -genkey -out ca.key
openssl req -new -x509 -days 36524 -key "ca.key" -sha384 -out ca.crt

openssl ecparam -name secp384r1 -genkey -out localhost.key
#openssl req -new -extensions v3_req -key "localhost.key" -sha384 -out localhost.csr
openssl req -new -nodes -key "localhost.key" -config csrconfig.txt -out localhost.csr

#openssl ca -key "ca.key" -in "localhost.csr" -out localhost.crt
#openssl ca -gencrl -keyfile ca.key -cert localhost.csr -out localhost.crt
openssl x509 -req  -extensions req_ext -extfile csrconfig.txt -in localhost.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out localhost.crt
#openssl x509 -req -signkey ca.key -in localhost.csr -out localhost.crt
```

## Running

Then, to run the test, run
```bash
./test.sh
```