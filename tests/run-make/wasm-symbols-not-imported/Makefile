include ../tools.mk

# only-wasm32-bare

all:
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown
	$(NODE) verify-no-imports.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -C lto
	$(NODE) verify-no-imports.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -O
	$(NODE) verify-no-imports.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -O -C lto
	$(NODE) verify-no-imports.js $(TMPDIR)/foo.wasm
