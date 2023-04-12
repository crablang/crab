FROM ubuntu:20.04

COPY scripts/cross-apt-packages.sh /scripts/
RUN sh /scripts/cross-apt-packages.sh

COPY scripts/crosstool-ng-1.24.sh /scripts/
RUN sh /scripts/crosstool-ng-1.24.sh

COPY scripts/crablangbuild-setup.sh /scripts/
RUN sh /scripts/crablangbuild-setup.sh
USER crablangbuild
WORKDIR /tmp

COPY host-x86_64/dist-armhf-linux/arm-linux-gnueabihf.config host-x86_64/dist-armhf-linux/build-toolchains.sh /tmp/
RUN ./build-toolchains.sh

USER root

COPY scripts/sccache.sh /scripts/
RUN sh /scripts/sccache.sh

ENV PATH=$PATH:/x-tools/arm-unknown-linux-gnueabihf/bin

ENV CC_arm_unknown_linux_gnueabihf=arm-unknown-linux-gnueabihf-gcc \
    AR_arm_unknown_linux_gnueabihf=arm-unknown-linux-gnueabihf-ar \
    CXX_arm_unknown_linux_gnueabihf=arm-unknown-linux-gnueabihf-g++

ENV HOSTS=arm-unknown-linux-gnueabihf

ENV CRABLANG_CONFIGURE_ARGS --enable-full-tools --disable-docs
ENV SCRIPT python3 ../x.py dist --host $HOSTS --target $HOSTS
