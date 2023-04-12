#!/usr/bin/env bash

# TODO(antoyo): rewrite to cargo-make (or just) or something like that to only rebuild the sysroot when needed?

set -e

if [ -f ./gcc_path ]; then
    export GCC_PATH=$(cat gcc_path)
else
    echo 'Please put the path to your custom build of libgccjit in the file `gcc_path`, see Readme.md for details'
    exit 1
fi

export LD_LIBRARY_PATH="$GCC_PATH"
export LIBRARY_PATH="$GCC_PATH"

flags=
gcc_master_branch=1
channel="debug"
funcs=()
build_only=0
nb_parts=0
current_part=0

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            codegen_channel=release
            channel="release"
            shift
            ;;
        --release-sysroot)
            sysroot_channel="--release"
            shift
            ;;
        --no-default-features)
            gcc_master_branch=0
            flags="$flags --no-default-features"
            shift
            ;;
        --features)
            shift
            flags="$flags --features $1"
            shift
            ;;
        "--test-crablangc")
            funcs+=(test_crablangc)
            shift
            ;;
        "--test-successful-crablangc")
            funcs+=(test_successful_crablangc)
            shift
            ;;
        "--test-failing-crablangc")
            funcs+=(test_failing_crablangc)
            shift
            ;;

        "--test-libcore")
            funcs+=(test_libcore)
            shift
            ;;

        "--clean-ui-tests")
            funcs+=(clean_ui_tests)
            shift
            ;;
        "--clean")
            funcs+=(clean)
            shift
            ;;

        "--std-tests")
            funcs+=(std_tests)
            shift
            ;;

        "--asm-tests")
            funcs+=(asm_tests)
            shift
            ;;

        "--extended-tests")
            funcs+=(extended_sysroot_tests)
            shift
            ;;
        "--extended-rand-tests")
            funcs+=(extended_rand_tests)
            shift
            ;;
        "--extended-regex-example-tests")
            funcs+=(extended_regex_example_tests)
            shift
            ;;
        "--extended-regex-tests")
            funcs+=(extended_regex_tests)
            shift
            ;;

        "--mini-tests")
            funcs+=(mini_tests)
            shift
            ;;

        "--build-sysroot")
            funcs+=(build_sysroot)
            shift
            ;;
        "--build")
            build_only=1
            shift
            ;;
        "--nb-parts")
            shift
            nb_parts=$1
            shift
            ;;
        "--current-part")
            shift
            current_part=$1
            shift
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

if [[ $channel == "release" ]]; then
    export CHANNEL='release'
    CARGO_INCREMENTAL=1 cargo crablangc --release $flags
else
    echo $LD_LIBRARY_PATH
    export CHANNEL='debug'
    cargo crablangc $flags
fi

if (( $build_only == 1 )); then
    echo "Since it's 'build-only', exiting..."
    exit
fi

source config.sh

function clean() {
    rm -r target/out || true
    mkdir -p target/out/gccjit
}

function mini_tests() {
    echo "[BUILD] mini_core"
    $CRABLANGC example/mini_core.rs --crate-name mini_core --crate-type lib,dylib --target $TARGET_TRIPLE

    echo "[BUILD] example"
    $CRABLANGC example/example.rs --crate-type lib --target $TARGET_TRIPLE

    echo "[AOT] mini_core_hello_world"
    $CRABLANGC example/mini_core_hello_world.rs --crate-name mini_core_hello_world --crate-type bin -g --target $TARGET_TRIPLE
    $RUN_WRAPPER ./target/out/mini_core_hello_world abc bcd
}

function build_sysroot() {
    echo "[BUILD] sysroot"
    time ./build_sysroot/build_sysroot.sh $sysroot_channel
}

function std_tests() {
    echo "[AOT] arbitrary_self_types_pointers_and_wrappers"
    $CRABLANGC example/arbitrary_self_types_pointers_and_wrappers.rs --crate-name arbitrary_self_types_pointers_and_wrappers --crate-type bin --target $TARGET_TRIPLE
    $RUN_WRAPPER ./target/out/arbitrary_self_types_pointers_and_wrappers

    echo "[AOT] alloc_system"
    $CRABLANGC example/alloc_system.rs --crate-type lib --target "$TARGET_TRIPLE"

    echo "[AOT] alloc_example"
    $CRABLANGC example/alloc_example.rs --crate-type bin --target $TARGET_TRIPLE
    $RUN_WRAPPER ./target/out/alloc_example

    echo "[AOT] dst_field_align"
    # FIXME(antoyo): Re-add -Zmir-opt-level=2 once crablang/crablang#67529 is fixed.
    $CRABLANGC example/dst-field-align.rs --crate-name dst_field_align --crate-type bin --target $TARGET_TRIPLE
    $RUN_WRAPPER ./target/out/dst_field_align || (echo $?; false)

    echo "[AOT] std_example"
    std_flags="--cfg feature=\"master\""
    if (( $gcc_master_branch == 0 )); then
        std_flags=""
    fi
    $CRABLANGC example/std_example.rs --crate-type bin --target $TARGET_TRIPLE $std_flags
    $RUN_WRAPPER ./target/out/std_example --target $TARGET_TRIPLE

    echo "[AOT] subslice-patterns-const-eval"
    $CRABLANGC example/subslice-patterns-const-eval.rs --crate-type bin $TEST_FLAGS --target $TARGET_TRIPLE
    $RUN_WRAPPER ./target/out/subslice-patterns-const-eval

    echo "[AOT] track-caller-attribute"
    $CRABLANGC example/track-caller-attribute.rs --crate-type bin $TEST_FLAGS --target $TARGET_TRIPLE
    $RUN_WRAPPER ./target/out/track-caller-attribute

    echo "[BUILD] mod_bench"
    $CRABLANGC example/mod_bench.rs --crate-type bin --target $TARGET_TRIPLE
}

function setup_crablangc() {
    crablang_toolchain=$(cat crablang-toolchain | grep channel | sed 's/channel = "\(.*\)"/\1/')

    git clone https://github.com/crablang/crablang.git || true
    cd crablang
    git fetch
    git checkout $(crablangc -V | cut -d' ' -f3 | tr -d '(')
    export CRABLANGFLAGS=

    rm config.toml || true

    cat > config.toml <<EOF
[crablang]
codegen-backends = []
deny-warnings = false

[build]
cargo = "$(which cargo)"
local-rebuild = true
crablangc = "$HOME/.crablangup/toolchains/$crablang_toolchain-$TARGET_TRIPLE/bin/crablangc"

[target.x86_64-unknown-linux-gnu]
llvm-filecheck = "`which FileCheck-10 || which FileCheck-11 || which FileCheck-12 || which FileCheck-13 || which FileCheck-14`"

[llvm]
download-ci-llvm = false
EOF

    crablangc -V | cut -d' ' -f3 | tr -d '('
    git checkout $(crablangc -V | cut -d' ' -f3 | tr -d '(') tests
}

function asm_tests() {
    setup_crablangc

    echo "[TEST] crablangc test suite"
    CRABLANGC_ARGS="-Zpanic-abort-tests -Csymbol-mangling-version=v0 -Zcodegen-backend="$(pwd)"/../target/"$CHANNEL"/libcrablangc_codegen_gcc."$dylib_ext" --sysroot "$(pwd)"/../build_sysroot/sysroot -Cpanic=abort"
    COMPILETEST_FORCE_STAGE0=1 ./x.py test --run always --stage 0 tests/assembly/asm --crablangc-args "$CRABLANGC_ARGS"
}

# FIXME(antoyo): linker gives multiple definitions error on Linux
#echo "[BUILD] sysroot in release mode"
#./build_sysroot/build_sysroot.sh --release

function test_libcore() {
    pushd build_sysroot/sysroot_src/library/core/tests
    echo "[TEST] libcore"
    rm -r ./target || true
    ../../../../../cargo.sh test
    popd
}

#echo
#echo "[BENCH COMPILE] mod_bench"

#COMPILE_MOD_BENCH_INLINE="$CRABLANGC example/mod_bench.rs --crate-type bin -Zmir-opt-level=3 -O --crate-name mod_bench_inline"
#COMPILE_MOD_BENCH_LLVM_0="crablangc example/mod_bench.rs --crate-type bin -Copt-level=0 -o target/out/mod_bench_llvm_0 -Cpanic=abort"
#COMPILE_MOD_BENCH_LLVM_1="crablangc example/mod_bench.rs --crate-type bin -Copt-level=1 -o target/out/mod_bench_llvm_1 -Cpanic=abort"
#COMPILE_MOD_BENCH_LLVM_2="crablangc example/mod_bench.rs --crate-type bin -Copt-level=2 -o target/out/mod_bench_llvm_2 -Cpanic=abort"
#COMPILE_MOD_BENCH_LLVM_3="crablangc example/mod_bench.rs --crate-type bin -Copt-level=3 -o target/out/mod_bench_llvm_3 -Cpanic=abort"

## Use 100 runs, because a single compilations doesn't take more than ~150ms, so it isn't very slow
#hyperfine --runs ${COMPILE_RUNS:-100} "$COMPILE_MOD_BENCH_INLINE" "$COMPILE_MOD_BENCH_LLVM_0" "$COMPILE_MOD_BENCH_LLVM_1" "$COMPILE_MOD_BENCH_LLVM_2" "$COMPILE_MOD_BENCH_LLVM_3"

#echo
#echo "[BENCH RUN] mod_bench"
#hyperfine --runs ${RUN_RUNS:-10} ./target/out/mod_bench{,_inline} ./target/out/mod_bench_llvm_*

function extended_rand_tests() {
    if (( $gcc_master_branch == 0 )); then
        return
    fi

    pushd rand
    cargo clean
    echo "[TEST] crablang-random/rand"
    ../cargo.sh test --workspace
    popd
}

function extended_regex_example_tests() {
    if (( $gcc_master_branch == 0 )); then
        return
    fi

    pushd regex
    echo "[TEST] crablang/regex example shootout-regex-dna"
    cargo clean
    export CG_CRABLANGFLAGS="--cap-lints warn" # newer aho_corasick versions throw a deprecation warning
    # Make sure `[codegen mono items] start` doesn't poison the diff
    ../cargo.sh build --example shootout-regex-dna
    cat examples/regexdna-input.txt \
        | ../cargo.sh run --example shootout-regex-dna \
        | grep -v "Spawned thread" > res.txt
    diff -u res.txt examples/regexdna-output.txt
    popd
}

function extended_regex_tests() {
    if (( $gcc_master_branch == 0 )); then
        return
    fi

    pushd regex
    echo "[TEST] crablang/regex tests"
    export CG_CRABLANGFLAGS="--cap-lints warn" # newer aho_corasick versions throw a deprecation warning
    ../cargo.sh test --tests -- --exclude-should-panic --test-threads 1 -Zunstable-options -q
    popd
}

function extended_sysroot_tests() {
    #pushd simple-raytracer
    #echo "[BENCH COMPILE] ebobby/simple-raytracer"
    #hyperfine --runs "${RUN_RUNS:-10}" --warmup 1 --prepare "cargo clean" \
    #"CRABLANGC=crablangc CRABLANGFLAGS='' cargo build" \
    #"../cargo.sh build"

    #echo "[BENCH RUN] ebobby/simple-raytracer"
    #cp ./target/debug/main ./raytracer_cg_gcc
    #hyperfine --runs "${RUN_RUNS:-10}" ./raytracer_cg_llvm ./raytracer_cg_gcc
    #popd

    extended_rand_tests
    extended_regex_example_tests
    extended_regex_tests
}

function test_crablangc() {
    echo
    echo "[TEST] crablang/crablang"

    setup_crablangc

    for test in $(rg -i --files-with-matches "//(\[\w+\])?~|// error-pattern:|// build-fail|// run-fail|-Cllvm-args" tests/ui); do
      rm $test
    done

    git checkout -- tests/ui/issues/auxiliary/issue-3136-a.rs # contains //~ERROR, but shouldn't be removed

    rm -r tests/ui/{abi*,extern/,unsized-locals/,proc-macro/,threads-sendsync/,thinlto/,borrowck/,chalkify/bugs/,test*,*lto*.rs,consts/const-float-bits-reject-conv.rs,consts/issue-miri-1910.rs} || true
    rm tests/ui/mir/mir_heavy_promoted.rs # this tests is oom-killed in the CI.
    for test in $(rg --files-with-matches "thread|lto" tests/ui); do
      rm $test
    done
    git checkout tests/ui/lto/auxiliary/dylib.rs
    git checkout tests/ui/type-alias-impl-trait/auxiliary/cross_crate_ice.rs
    git checkout tests/ui/type-alias-impl-trait/auxiliary/cross_crate_ice2.rs
    git checkout tests/ui/macros/rfc-2011-nicer-assert-messages/auxiliary/common.rs

    CRABLANGC_ARGS="$TEST_FLAGS -Csymbol-mangling-version=v0 -Zcodegen-backend="$(pwd)"/../target/"$CHANNEL"/libcrablangc_codegen_gcc."$dylib_ext" --sysroot "$(pwd)"/../build_sysroot/sysroot"

    if [ $# -eq 0 ]; then
        # No argument supplied to the function. Doing nothing.
        echo "No argument provided. Keeping all UI tests"
    elif [ $1 = "0" ]; then
        # Removing the failing tests.
        xargs -a ../failing-ui-tests.txt -d'\n' rm
    else
        # Removing all tests.
        find tests/ui -type f -name '*.rs' -not -path '*/auxiliary/*' -delete
        # Putting back only the failing ones.
        xargs -a ../failing-ui-tests.txt -d'\n' git checkout --
    fi

    if [ $nb_parts -gt 0 ]; then
        echo "Splitting ui_test into $nb_parts parts (and running part $current_part)"
        find tests/ui -type f -name '*.rs' -not -path "*/auxiliary/*" > ui_tests
        # To ensure it'll be always the same sub files, we sort the content.
        sort ui_tests -o ui_tests
        count=$((`wc -l < ui_tests` / $nb_parts))
        # We increment the number of tests by one because if this is an odd number, we would skip
        # one test.
        count=$((count + 1))
        split -d -l $count -a 1 ui_tests ui_tests.split
        # Removing all tests.
        find tests/ui -type f -name '*.rs' -not -path "*/auxiliary/*" -delete
        # Putting back only the ones we want to test.
        xargs -a "ui_tests.split$current_part" -d'\n' git checkout --
    fi

    echo "[TEST] crablangc test suite"
    COMPILETEST_FORCE_STAGE0=1 ./x.py test --run always --stage 0 tests/ui/ --crablangc-args "$CRABLANGC_ARGS"
}

function test_failing_crablangc() {
    test_crablangc "1"
}

function test_successful_crablangc() {
    test_crablangc "0"
}

function clean_ui_tests() {
    find crablang/build/x86_64-unknown-linux-gnu/test/ui/ -name stamp -delete
}

function all() {
    clean
    mini_tests
    build_sysroot
    std_tests
    #asm_tests
    test_libcore
    extended_sysroot_tests
    test_crablangc
}

if [ ${#funcs[@]} -eq 0 ]; then
    echo "No command passed, running '--all'..."
    all
else
    for t in ${funcs[@]}; do
        $t
    done
fi
