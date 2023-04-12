include ../../run-make/tools.mk

# This test ensures that crablangc compile_input can be called twice in one task
# without causing a panic.
# The program needs the path to crablangc to get sysroot.

all:
	$(CRABLANGC) foo.rs
	$(call RUN,foo $(TMPDIR) $(CRABLANGC))
