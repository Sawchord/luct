# Build instructions

### Setup

The easiest way to build the luCT firefox extension is to use the supplied `Dockerfile`.

You need to have docker with `buildx` support installed.
Please follow the [docker install instructions](https://docs.docker.com/engine/install/) for your platform.

If you are on Ubuntu, the install instructions can be found [here](https://docs.docker.com/engine/install/ubuntu/).


It might also be necessary to add you current user to the `docker` group to enable non-root usage:

```
sudo usermod -a -G docker $USER
```

### Build

To build the extension, from the top-level directory of the project run:

```
docker buildx build --progress=plain . -o .
```

## Building the `aarch64` version on `x64_86`

It is currently not possible to have a reproducible build across CPU architectures.
This is due to the rust compiler not being able to produce the same WASM.

If you want to build the extension on an `x86_64` machine but you want to get the same artifacts as if it was build on `aarch64`,
you need to run the build inside qemu.

According to [this source](https://extensionworkshop.com/documentation/publish/source-code-submission/#default-reviewer-build-environment)
the reviwers use "ARM64" builders.

Using this setup should create the same aritfacts as their build environment.


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