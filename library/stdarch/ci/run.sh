#!/usr/bin/env sh

set -ex

: "${TARGET?The TARGET environment variable must be set.}"

# Tests are all super fast anyway, and they fault often enough on travis that
# having only one thread increases debuggability to be worth it.
#export RUST_BACKTRACE=full
#export RUST_TEST_NOCAPTURE=1
#export RUST_TEST_THREADS=1

export RUSTFLAGS="${RUSTFLAGS} -D warnings -Z merge-functions=disabled "
export HOST_RUSTFLAGS="${RUSTFLAGS}"

export STDARCH_DISABLE_DEDUP_GUARD=1

case ${TARGET} in
    # On Windows the linker performs identical COMDAT folding (ICF) by default
    # in release mode which removes identical COMDAT sections. This interferes
    # with our instruction assertions just like LLVM's MergeFunctions pass so
    # we disable it.
    *-pc-windows-msvc)
        export RUSTFLAGS="${RUSTFLAGS} -Clink-args=/OPT:NOICF"
        ;;
    # On 32-bit use a static relocation model which avoids some extra
    # instructions when dealing with static data, notably allowing some
    # instruction assertion checks to pass below the 20 instruction limit. If
    # this is the default, dynamic, then too many instructions are generated
    # when we assert the instruction for a function and it causes tests to fail.
    #
    # It's not clear why `-Z plt=yes` is required here. Probably a bug in LLVM.
    # If you can remove it and CI passes, please feel free to do so!
    i686-* | i586-*)
        export RUSTFLAGS="${RUSTFLAGS} -C relocation-model=static -Z plt=yes"
        ;;
    # Some x86_64 targets enable by default more features beyond SSE2,
    # which cause some instruction assertion checks to fail.
    x86_64-*)
        export RUSTFLAGS="${RUSTFLAGS} -C target-feature=-sse3"
        ;;
    #Unoptimized build uses fast-isel which breaks with msa
    mips-* | mipsel-*)
	export RUSTFLAGS="${RUSTFLAGS} -C llvm-args=-fast-isel=false"
	;;
    # Some of our test dependencies use the deprecated `gcc` crates which is
    # missing a fix from https://github.com/alexcrichton/cc-rs/pull/627. Apply
    # the workaround manually here.
    armv7-*eabihf | thumbv7-*eabihf)
        export RUSTFLAGS="${RUSTFLAGS} -Ctarget-feature=+neon"
        export TARGET_CFLAGS="-mfpu=vfpv3-d16"
        ;;
    # Some of our test dependencies use the deprecated `gcc` crates which
    # doesn't detect RISC-V compilers automatically, so do it manually here.
    riscv64*)
        export RUSTFLAGS="${RUSTFLAGS} -Ctarget-feature=+zk,+zks,+zbb,+zbc"
        export TARGET_CC="riscv64-linux-gnu-gcc"
        ;;
esac

echo "RUSTFLAGS=${RUSTFLAGS}"
echo "FEATURES=${FEATURES}"
echo "OBJDUMP=${OBJDUMP}"
echo "STDARCH_DISABLE_ASSERT_INSTR=${STDARCH_DISABLE_ASSERT_INSTR}"
echo "STDARCH_TEST_EVERYTHING=${STDARCH_TEST_EVERYTHING}"

cargo_test() {
    cmd="cargo"
    subcmd="test"
    if [ "$NORUN" = "1" ]; then
        export subcmd="build"
    fi
    cmd="$cmd ${subcmd} --target=$TARGET $1"
    cmd="$cmd -- $2"

    # wasm targets can't catch panics so if a test failures make sure the test
    # harness isn't trying to capture output, otherwise we won't get any useful
    # output.
    case ${TARGET} in
        wasm32*)
            cmd="$cmd --nocapture"
            ;;
        # qemu has an erratic behavior on those tests
        powerpc64*)
            cmd="$cmd --skip test_vec_lde_u16 --skip test_vec_lde_u32 --skip test_vec_expte"
            ;;
        # Miscompilation: https://github.com/rust-lang/rust/issues/112460
        arm*)
            cmd="$cmd --skip vld2q_dup_f32"
            ;;
    esac

    if [ "$SKIP_TESTS" != "" ]; then
        cmd="$cmd --skip "$SKIP_TESTS
    fi
    $cmd
}

CORE_ARCH="--manifest-path=crates/core_arch/Cargo.toml"
STD_DETECT="--manifest-path=crates/std_detect/Cargo.toml"
STDARCH_EXAMPLES="--manifest-path=examples/Cargo.toml"
INTRINSIC_TEST="--manifest-path=crates/intrinsic-test/Cargo.toml"

cargo_test "${CORE_ARCH} --release"

if [ "$NOSTD" != "1" ]; then
    cargo_test "${STD_DETECT}"
    cargo_test "${STD_DETECT} --release"

    cargo_test "${STD_DETECT} --no-default-features"
    cargo_test "${STD_DETECT} --no-default-features --features=std_detect_file_io"
    cargo_test "${STD_DETECT} --no-default-features --features=std_detect_dlsym_getauxval"
    cargo_test "${STD_DETECT} --no-default-features --features=std_detect_dlsym_getauxval,std_detect_file_io"

    cargo_test "${STDARCH_EXAMPLES}"
    cargo_test "${STDARCH_EXAMPLES} --release"
fi

# Test targets compiled with extra features.
case ${TARGET} in
    x86*)
        export STDARCH_DISABLE_ASSERT_INSTR=1
        export RUSTFLAGS="${RUSTFLAGS} -C target-feature=+avx"
        cargo_test "--release"
        ;;
    # FIXME: don't build anymore
    #mips-*gnu* | mipsel-*gnu*)
    #    export RUSTFLAGS="${RUSTFLAGS} -C target-feature=+msa,+fp64,+mips32r5"
    #    cargo_test "--release"
	  #    ;;
    mips64*)
        export RUSTFLAGS="${RUSTFLAGS} -C target-feature=+msa"
        cargo_test "--release"
	      ;;
    powerpc64*)
        # We don't build the ppc 32-bit targets with these - these targets
        # are mostly unsupported for now.
        OLD_RUSTFLAGS="${RUSTFLAGS}"
        export RUSTFLAGS="${OLD_RUSTFLAGS} -C target-feature=+altivec"
        cargo_test "--release"

        export RUSTFLAGS="${OLD_RUSTFLAGS} -C target-feature=+vsx"
        cargo_test "--release"
        ;;
    *)
        ;;

esac

if [ "${TARGET}" = "aarch64-unknown-linux-gnu" ]; then
    (
        CPPFLAGS="-fuse-ld=lld -I/usr/aarch64-linux-gnu/include/ -I/usr/aarch64-linux-gnu/include/c++/9/aarch64-linux-gnu/" \
            RUSTFLAGS="$HOST_RUSTFLAGS" \
            RUST_LOG=warn \
            cargo run ${INTRINSIC_TEST} --release --bin intrinsic-test -- intrinsics_data/arm_intrinsics.json --runner "${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER}" --cppcompiler "clang++-15" --skip crates/intrinsic-test/missing_aarch64.txt
    )
elif [ "${TARGET}" = "armv7-unknown-linux-gnueabihf" ]; then
    (
        CPPFLAGS="-fuse-ld=lld -I/usr/arm-linux-gnueabihf/include/ -I/usr/arm-linux-gnueabihf/include/c++/9/arm-linux-gnueabihf/" \
            RUSTFLAGS="$HOST_RUSTFLAGS" \
            RUST_LOG=warn \
            cargo run ${INTRINSIC_TEST} --release --bin intrinsic-test -- intrinsics_data/arm_intrinsics.json --runner "${CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUNNER}" --cppcompiler "clang++-15" --skip crates/intrinsic-test/missing_arm.txt --a32
    )
fi

if [ "$NORUN" != "1" ] && [ "$NOSTD" != 1 ]; then
    # Test examples
    (
        cd examples
        cargo test --target "$TARGET"
        echo test | cargo run --target "$TARGET" --release hex
    )
fi
