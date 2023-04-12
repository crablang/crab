FROM ubuntu:20.04

COPY scripts/cross-apt-packages.sh /scripts/
RUN sh /scripts/cross-apt-packages.sh

COPY scripts/crosstool-ng-1.24.sh /scripts/
RUN sh /scripts/crosstool-ng-1.24.sh

COPY scripts/crablangbuild-setup.sh /scripts/
RUN sh /scripts/crablangbuild-setup.sh
USER crablangbuild
WORKDIR /tmp

COPY host-x86_64/dist-powerpc-linux/powerpc-linux-gnu.config host-x86_64/dist-powerpc-linux/build-powerpc-toolchain.sh /tmp/
RUN ./build-powerpc-toolchain.sh

USER root

COPY scripts/sccache.sh /scripts/
RUN sh /scripts/sccache.sh

ENV PATH=$PATH:/x-tools/powerpc-unknown-linux-gnu/bin

ENV \
    CC_powerpc_unknown_linux_gnu=powerpc-unknown-linux-gnu-gcc \
    AR_powerpc_unknown_linux_gnu=powerpc-unknown-linux-gnu-ar \
    CXX_powerpc_unknown_linux_gnu=powerpc-unknown-linux-gnu-g++

ENV HOSTS=powerpc-unknown-linux-gnu

ENV CRABLANG_CONFIGURE_ARGS --enable-extended --disable-docs
ENV SCRIPT python3 ../x.py dist --host $HOSTS --target $HOSTS
