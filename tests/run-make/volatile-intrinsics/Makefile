# ignore-cross-compile
include ../tools.mk

all:
	# The tests must pass...
	$(CRABLANGC) main.rs
	$(call RUN,main)
	# ... and the loads/stores must not be optimized out.
	$(CRABLANGC) main.rs --emit=llvm-ir
	$(CGREP) "load volatile" "store volatile" < $(TMPDIR)/main.ll
