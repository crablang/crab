include ../tools.mk

OUTPUT_DIR := "$(TMPDIR)/crablangdoc"

$(TMPDIR)/%.calls: $(TMPDIR)/libfoobar.rmeta
	$(CRABLANGDOC) examples/$*.rs --crate-name $* --crate-type bin --output $(OUTPUT_DIR) \
	  --extern foobar=$(TMPDIR)/libfoobar.rmeta \
		-Z unstable-options \
		--scrape-examples-output-path $@ \
		--scrape-examples-target-crate foobar \
		$(extra_flags)

$(TMPDIR)/lib%.rmeta: src/lib.rs
	$(CRABLANGC) src/lib.rs --crate-name $* --crate-type lib --emit=metadata

scrape: $(foreach d,$(deps),$(TMPDIR)/$(d).calls)
	$(CRABLANGDOC) src/lib.rs --crate-name foobar --crate-type lib --output $(OUTPUT_DIR) \
		-Z unstable-options \
		$(foreach d,$(deps),--with-examples $(TMPDIR)/$(d).calls)

	$(HTMLDOCCK) $(OUTPUT_DIR) src/lib.rs
