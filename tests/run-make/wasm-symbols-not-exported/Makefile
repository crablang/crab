include ../tools.mk

# only-wasm32-bare

all:
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown
	$(NODE) verify-exported-symbols.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -O
	$(NODE) verify-exported-symbols.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown
	$(NODE) verify-exported-symbols.js $(TMPDIR)/bar.wasm
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown -O
	$(NODE) verify-exported-symbols.js $(TMPDIR)/bar.wasm
