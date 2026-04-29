# Build instructions

TODO

## Building the `aarch64` version on `x64_86`

It is currently not possible to have a reproducible build across CPU architectures.
This is due to the rust compiler not being able to produce the same WASM.

If you want to build the extension on an `x86_64` machine but you want to get the same artifacts as if it was build on `aarch64`,
you need to run the build inside qemu.

According to [this source](https://extensionworkshop.com/documentation/publish/source-code-submission/#default-reviewer-build-environment)
the reviwers use "ARM64" builders.

Using this setup creates the same aritfacts as the CI pipeline.


### Setup

To setup docker to use `qemu-aarch64` when building, run:

```
docker run --rm --privileged tonistiigi/binfmt --install all
```

You can check that `qemu-arch64` is correctly enabled by running

```
cat /proc/sys/fs/binfmt_misc/qemu-aarch64
```

you should see the following result:

```
enabled
interpreter /usr/bin/qemu-aarch64
flags: POCF
offset 0
magic 7f454c460201010000000000000000000200b700
mask ffffffffffffff00fffffffffffffffffeffffff
```

### Build

```
docker buildx build --platform=linux/arm64 --progress=plain . -o .
```