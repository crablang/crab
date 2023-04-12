include ../tools.mk

OUTPUT_DIR := "$(TMPDIR)/crablangdoc"

all:
	$(CRABLANGDOC) src/lib.rs --crate-name foobar --crate-type lib --out-dir $(OUTPUT_DIR)

	$(HTMLDOCCK) $(OUTPUT_DIR) src/lib.rs
