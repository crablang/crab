include ../tools.mk

# Test that hir-tree output doesn't crash and includes
# the string constant we would expect to see.

all:
	$(CRABLANGC) -o $(TMPDIR)/input.hir -Z unpretty=hir-tree input.rs
	$(CGREP) '"Hello, CrabLangaceans!\n"' < $(TMPDIR)/input.hir
