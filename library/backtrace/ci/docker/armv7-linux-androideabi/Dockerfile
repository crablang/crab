FROM ubuntu:20.04

RUN apt-get update && apt-get install -y --no-install-recommends \
  curl \
  ca-certificates \
  unzip \
  openjdk-8-jre \
  python \
  gcc \
  libc6-dev

COPY android-ndk.sh /
RUN /android-ndk.sh
ENV PATH=$PATH:/android-toolchain/ndk/toolchains/llvm/prebuilt/linux-x86_64/bin

# TODO: run tests in an emulator eventually
ENV CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=armv7a-linux-androideabi19-clang \
    CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_RUNNER=echo
