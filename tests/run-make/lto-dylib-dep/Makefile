# ignore-cross-compile
include ../tools.mk

# Test that we don't run into an assertion when using a CrabLang dylib dependency
# while compiling with full LTO.
# See https://github.com/crablang/crablang/issues/59137

all:
	$(CRABLANGC) a_dylib.rs --crate-type=dylib -C prefer-dynamic
	$(CRABLANGC) main.rs -C lto
	$(call RUN,main)
