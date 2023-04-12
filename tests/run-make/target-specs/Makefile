include ../tools.mk
all:
	$(CRABLANGC) foo.rs --target=my-awesome-platform.json --crate-type=lib --emit=asm
	$(CGREP) -v morestack < $(TMPDIR)/foo.s
	$(CRABLANGC) foo.rs --target=my-invalid-platform.json 2>&1 | $(CGREP) "Error loading target specification"
	$(CRABLANGC) foo.rs --target=my-incomplete-platform.json 2>&1 | $(CGREP) 'Field llvm-target'
	CRABLANG_TARGET_PATH=. $(CRABLANGC) foo.rs --target=my-awesome-platform --crate-type=lib --emit=asm
	CRABLANG_TARGET_PATH=. $(CRABLANGC) foo.rs --target=my-x86_64-unknown-linux-gnu-platform --crate-type=lib --emit=asm
	$(CRABLANGC) -Z unstable-options --target=my-awesome-platform.json --print target-spec-json > $(TMPDIR)/test-platform.json && $(CRABLANGC) -Z unstable-options --target=$(TMPDIR)/test-platform.json --print target-spec-json | diff -q $(TMPDIR)/test-platform.json -
	$(CRABLANGC) foo.rs --target=definitely-not-builtin-target 2>&1 | $(CGREP) 'may not set is_builtin'
	$(CRABLANGC) foo.rs --target=mismatching-data-layout --crate-type=lib
