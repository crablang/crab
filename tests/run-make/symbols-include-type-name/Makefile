include ../tools.mk

# Check that symbol names for methods include type names, instead of <impl>.

OUT=$(TMPDIR)/lib.s

all:
	$(CRABLANGC) --crate-type staticlib --emit asm lib.rs
	$(CGREP) Def < $(OUT)
