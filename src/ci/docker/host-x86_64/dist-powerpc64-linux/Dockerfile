FROM ubuntu:20.04

COPY scripts/cross-apt-packages.sh /scripts/
RUN sh /scripts/cross-apt-packages.sh

COPY scripts/crosstool-ng-1.24.sh /scripts/
RUN sh /scripts/crosstool-ng-1.24.sh

COPY scripts/crablangbuild-setup.sh /scripts/
RUN sh /scripts/crablangbuild-setup.sh
USER crablangbuild
WORKDIR /tmp

COPY host-x86_64/dist-powerpc64-linux/shared.sh host-x86_64/dist-powerpc64-linux/powerpc64-linux-gnu.config host-x86_64/dist-powerpc64-linux/build-powerpc64-toolchain.sh /tmp/
RUN ./build-powerpc64-toolchain.sh

USER root

COPY scripts/sccache.sh /scripts/
RUN sh /scripts/sccache.sh

ENV PATH=$PATH:/x-tools/powerpc64-unknown-linux-gnu/bin

ENV \
    AR_powerpc64_unknown_linux_gnu=powerpc64-unknown-linux-gnu-ar \
    CC_powerpc64_unknown_linux_gnu=powerpc64-unknown-linux-gnu-gcc \
    CXX_powerpc64_unknown_linux_gnu=powerpc64-unknown-linux-gnu-g++

ENV HOSTS=powerpc64-unknown-linux-gnu

ENV CRABLANG_CONFIGURE_ARGS --enable-extended --disable-docs
ENV SCRIPT python3 ../x.py dist --host $HOSTS --target $HOSTS
