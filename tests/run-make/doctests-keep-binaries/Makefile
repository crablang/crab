# ignore-cross-compile
include ../tools.mk

# Check that valid binaries are persisted by running them, regardless of whether the --run or --no-run option is used.

all: run no_run

run:
	mkdir -p $(TMPDIR)/doctests
	$(CRABLANGC) --crate-type rlib t.rs
	$(CRABLANGDOC) -Zunstable-options --test --persist-doctests $(TMPDIR)/doctests --extern t=$(TMPDIR)/libt.rlib t.rs
	$(TMPDIR)/doctests/t_rs_2_0/crablang_out
	$(TMPDIR)/doctests/t_rs_8_0/crablang_out
	rm -rf $(TMPDIR)/doctests

no_run:
	mkdir -p $(TMPDIR)/doctests
	$(CRABLANGC) --crate-type rlib t.rs
	$(CRABLANGDOC) -Zunstable-options --test --persist-doctests $(TMPDIR)/doctests --extern t=$(TMPDIR)/libt.rlib t.rs --no-run
	$(TMPDIR)/doctests/t_rs_2_0/crablang_out
	$(TMPDIR)/doctests/t_rs_8_0/crablang_out
	rm -rf $(TMPDIR)/doctests
