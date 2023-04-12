# ignore-cross-compile
include ../tools.mk

all: foo.rs
	$(CRABLANGC) --cfg 'feature="bar"' --crate-type lib foo.rs
	$(CRABLANGDOC) --test --cfg 'feature="bar"' \
		-L $(TMPDIR) foo.rs |\
		$(CGREP) 'foo.rs - foo (line 1) ... ok'
