include ../tools.mk

all:
	$(CRABLANGC) ep-nested-lib.rs

	$(CRABLANGC) use-suggestions.rs --edition=2018 --extern ep_nested_lib=$(TMPDIR)/libep_nested_lib.rlib 2>&1 | $(CGREP) "use ep_nested_lib::foo::bar::Baz"

