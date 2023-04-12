# ignore-cross-compile
include ../tools.mk

# ignore-windows

NAME := $(shell $(CRABLANGC) --print file-names foo.rs)

all:
	mkdir -p $(TMPDIR)/outdir
	$(CRABLANGC) foo.rs -o $(TMPDIR)/outdir/$(NAME)
	ln -nsf outdir/$(NAME) $(TMPDIR)
	CRABLANGC_LOG=crablangc_metadata::loader $(CRABLANGC) bar.rs
