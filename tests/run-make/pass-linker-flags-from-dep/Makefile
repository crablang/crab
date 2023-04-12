include ../tools.mk

all:
	# Build deps
	$(CRABLANGC) native_dep_1.rs --crate-type=staticlib
	$(CRABLANGC) native_dep_2.rs --crate-type=staticlib
	$(CRABLANGC) crablang_dep.rs -l static:-bundle=native_dep_1 -l link-arg=some_flag -l static:-bundle=native_dep_2 --crate-type=lib -Z unstable-options

	# Check sequence of linker args
	$(CRABLANGC) main.rs --extern lib=$(TMPDIR)/libcrablang_dep.rlib --crate-type=bin --print link-args | $(CGREP) -e 'native_dep_1.*some_flag.*native_dep_2'
