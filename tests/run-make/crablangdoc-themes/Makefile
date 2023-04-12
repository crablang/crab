include ../tools.mk

# Test that crablangdoc will properly load in a theme file and display it in the theme selector.

OUTPUT_DIR := "$(TMPDIR)/crablangdoc-themes"

all:
	cp $(S)/src/libcrablangdoc/html/static/css/themes/light.css $(TMPDIR)/test.css
	$(CRABLANGDOC) -o $(OUTPUT_DIR) foo.rs --theme $(TMPDIR)/test.css
	$(HTMLDOCCK) $(OUTPUT_DIR) foo.rs
