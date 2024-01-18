FROM ubuntu:20.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    ca-certificates \
    libc6-dev \
    gcc-s390x-linux-gnu \
    libc6-dev-s390x-cross \
    qemu-user \
    # There seems to be a bug in processing mixed-architecture
    # ld.so.cache files that causes crashes in some cases.  Work
    # around this by simply deleting the cache for now.
    && rm /etc/ld.so.cache

ENV CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER=s390x-linux-gnu-gcc \
    CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUNNER="qemu-s390x -L /usr/s390x-linux-gnu" \
    CC=s390x-linux-gnu-gcc
