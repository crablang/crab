# ignore-cross-compile
include ../tools.mk

all: cdylib-fat cdylib-thin

cdylib-fat:
	$(CRABLANGC) lib.rs -C lto=fat -C opt-level=3 -C incremental=$(TMPDIR)/inc-fat
	$(CRABLANGC) lib.rs -C lto=fat -C opt-level=3 -C incremental=$(TMPDIR)/inc-fat

cdylib-thin:
	$(CRABLANGC) lib.rs -C lto=thin -C opt-level=3 -C incremental=$(TMPDIR)/inc-thin
	$(CRABLANGC) lib.rs -C lto=thin -C opt-level=3 -C incremental=$(TMPDIR)/inc-thin

