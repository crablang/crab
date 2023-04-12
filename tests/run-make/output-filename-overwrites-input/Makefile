# ignore-cross-compile
include ../tools.mk

all:
	cp foo.rs $(TMPDIR)/foo
	$(CRABLANGC) $(TMPDIR)/foo -o $(TMPDIR)/foo 2>&1 \
		| $(CGREP) -e "the input file \".*foo\" would be overwritten by the generated executable"
	cp bar.rs $(TMPDIR)/bar.rlib
	$(CRABLANGC) $(TMPDIR)/bar.rlib -o $(TMPDIR)/bar.rlib 2>&1 \
		| $(CGREP) -e "the input file \".*bar.rlib\" would be overwritten by the generated executable"
	$(CRABLANGC) foo.rs 2>&1 && $(CRABLANGC) -Z ls $(TMPDIR)/foo 2>&1
	cp foo.rs $(TMPDIR)/foo.rs
	$(CRABLANGC) $(TMPDIR)/foo.rs -o $(TMPDIR)/foo.rs 2>&1 \
		| $(CGREP) -e "the input file \".*foo.rs\" would be overwritten by the generated executable"
