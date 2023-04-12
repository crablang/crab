include ../tools.mk

# only-wasm32-bare

all:
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown
	$(NODE) verify.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) main.rs --target wasm32-unknown-unknown
	$(NODE) verify.js $(TMPDIR)/main.wasm
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown -O
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -O
	$(NODE) verify.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) main.rs --target wasm32-unknown-unknown -O
	$(NODE) verify.js $(TMPDIR)/main.wasm
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -C lto
	$(NODE) verify.js $(TMPDIR)/foo.wasm
	$(CRABLANGC) main.rs --target wasm32-unknown-unknown -C lto
	$(NODE) verify.js $(TMPDIR)/main.wasm
