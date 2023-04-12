include ../tools.mk

# Test that crablangdoc will properly canonicalize the target spec json path just like crablangc

OUTPUT_DIR := "$(TMPDIR)/crablangdoc-target-spec-json-path"

all:
	$(CRABLANGC) --crate-type lib dummy_core.rs --target target.json
	$(CRABLANGDOC) -o $(OUTPUT_DIR) -L $(TMPDIR) my_crate.rs --target target.json
