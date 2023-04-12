# needs-matching-clang

# This test makes sure that cross-language inlining actually works by checking
# the generated machine code.

include ../tools.mk

all: cpp-executable crablang-executable

cpp-executable:
	$(CRABLANGC) -Clinker-plugin-lto=on -o $(TMPDIR)/libcrablanglib-xlto.a -Copt-level=2 -Ccodegen-units=1 ./crablanglib.rs
	$(CLANG) -flto=thin -fuse-ld=lld -L $(TMPDIR) -lcrablanglib-xlto -o $(TMPDIR)/cmain ./cmain.c -O3
	# Make sure we don't find a call instruction to the function we expect to
	# always be inlined.
	"$(LLVM_BIN_DIR)"/llvm-objdump -d $(TMPDIR)/cmain | $(CGREP) -v -e "call.*crablang_always_inlined"
	# As a sanity check, make sure we do find a call instruction to a
	# non-inlined function
	"$(LLVM_BIN_DIR)"/llvm-objdump -d $(TMPDIR)/cmain | $(CGREP) -e "call.*crablang_never_inlined"

crablang-executable:
	$(CLANG) ./clib.c -flto=thin -c -o $(TMPDIR)/clib.o -O2
	(cd $(TMPDIR); $(AR) crus ./libxyz.a ./clib.o)
	$(CRABLANGC) -Clinker-plugin-lto=on -L$(TMPDIR) -Copt-level=2 -Clinker=$(CLANG) -Clink-arg=-fuse-ld=lld ./main.rs -o $(TMPDIR)/rsmain
	"$(LLVM_BIN_DIR)"/llvm-objdump -d $(TMPDIR)/rsmain | $(CGREP) -e "call.*c_never_inlined"
	"$(LLVM_BIN_DIR)"/llvm-objdump -d $(TMPDIR)/rsmain | $(CGREP) -v -e "call.*c_always_inlined"
