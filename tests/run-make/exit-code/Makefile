# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) success.rs; [ $$? -eq 0 ]
	$(CRABLANGC) --invalid-arg-foo; [ $$? -eq 1 ]
	$(CRABLANGC) compile-error.rs; [ $$? -eq 1 ]
	$(CRABLANGC) -Ztreat-err-as-bug compile-error.rs; [ $$? -eq 101 ]
	$(CRABLANGDOC) -o $(TMPDIR)/exit-code success.rs; [ $$? -eq 0 ]
	$(CRABLANGDOC) --invalid-arg-foo; [ $$? -eq 1 ]
	$(CRABLANGDOC) compile-error.rs; [ $$? -eq 1 ]
	$(CRABLANGDOC) lint-failure.rs; [ $$? -eq 1 ]
