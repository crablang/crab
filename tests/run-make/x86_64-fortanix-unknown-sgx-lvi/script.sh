#!/bin/sh
set -exuo pipefail

function build {
    CRATE=enclave

    mkdir -p $WORK_DIR
    pushd $WORK_DIR
        rm -rf $CRATE
        cp -a $TEST_DIR/enclave .
        pushd $CRATE
            echo ${WORK_DIR}
            # HACK(eddyb) sets `CRABLANGC_BOOTSTRAP=1` so Cargo can accept nightly features.
            # These come from the top-level CrabLang workspace, that this crate is not a
            # member of, but Cargo tries to load the workspace `Cargo.toml` anyway.
            env CRABLANGC_BOOTSTRAP=1
                cargo -v run --target $TARGET
        popd
    popd
}

function check {
    local func_re="$1"
    local checks="${TEST_DIR}/$2"
    local asm=$(mktemp)
    local objdump="${LLVM_BIN_DIR}/llvm-objdump"
    local filecheck="${LLVM_BIN_DIR}/FileCheck"
    local enclave=${WORK_DIR}/enclave/target/x86_64-fortanix-unknown-sgx/debug/enclave

    func="$(${objdump} --syms --demangle ${enclave} | \
            grep --only-matching -E "[[:blank:]]+${func_re}\$" | \
            sed -e 's/^[[:space:]]*//' )"
    ${objdump} --disassemble-symbols="${func}" --demangle \
      ${enclave} > ${asm}
    ${filecheck} --input-file ${asm} ${checks}
}

build

check "unw_getcontext" unw_getcontext.checks
check "__libunwind_Registers_x86_64_jumpto" jumpto.checks
check 'std::io::stdio::_print::[[:alnum:]]+' print.checks
check crablang_plus_one_global_asm crablang_plus_one_global_asm.checks

check cc_plus_one_c cc_plus_one_c.checks
check cc_plus_one_c_asm cc_plus_one_c_asm.checks
check cc_plus_one_cxx cc_plus_one_cxx.checks
check cc_plus_one_cxx_asm cc_plus_one_cxx_asm.checks
check cc_plus_one_asm cc_plus_one_asm.checks

check cmake_plus_one_c cmake_plus_one_c.checks
check cmake_plus_one_c_asm cmake_plus_one_c_asm.checks
check cmake_plus_one_c_global_asm cmake_plus_one_c_global_asm.checks
check cmake_plus_one_cxx cmake_plus_one_cxx.checks
check cmake_plus_one_cxx_asm cmake_plus_one_cxx_asm.checks
check cmake_plus_one_cxx_global_asm cmake_plus_one_cxx_global_asm.checks
check cmake_plus_one_asm cmake_plus_one_asm.checks
