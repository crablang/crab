include ../tools.mk

OUTPUT_DIR := "$(TMPDIR)/crablangdoc"

all:
	$(CRABLANGDOC) src/lib.rs --crate-name foobar --crate-type lib -o $(OUTPUT_DIR)

	$(HTMLDOCCK) $(OUTPUT_DIR) src/lib.rs
