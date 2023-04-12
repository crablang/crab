# ignore-cross-compile
# ignore-macos

include ../tools.mk

all:
	# Verbatim allows specify precise name.
	$(CRABLANGC) local_native_dep.rs --crate-type=staticlib -o $(TMPDIR)/local_some_strange_name.ext
	$(CRABLANGC) main.rs -l static:+verbatim=local_some_strange_name.ext

	# With verbatim any other name cannot be used (local).
	$(CRABLANGC) local_native_dep.rs --crate-type=staticlib -o $(TMPDIR)/liblocal_native_dep.a
	$(CRABLANGC) local_native_dep.rs --crate-type=staticlib -o $(TMPDIR)/local_native_dep.a
	$(CRABLANGC) local_native_dep.rs --crate-type=staticlib -o $(TMPDIR)/local_native_dep.lib
	$(CRABLANGC) main.rs -l static:+verbatim=local_native_dep 2>&1 | $(CGREP) "local_native_dep"
