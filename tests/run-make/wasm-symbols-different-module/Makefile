include ../tools.mk

# only-wasm32-bare

all:
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown
	$(NODE) verify-imports.js $(TMPDIR)/foo.wasm a/foo b/foo
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -C lto
	$(NODE) verify-imports.js $(TMPDIR)/foo.wasm a/foo b/foo
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -O
	$(NODE) verify-imports.js $(TMPDIR)/foo.wasm a/foo b/foo
	$(CRABLANGC) foo.rs --target wasm32-unknown-unknown -O -C lto
	$(NODE) verify-imports.js $(TMPDIR)/foo.wasm a/foo b/foo

	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown
	$(NODE) verify-imports.js $(TMPDIR)/bar.wasm m1/f m1/g m2/f
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown -C lto
	$(NODE) verify-imports.js $(TMPDIR)/bar.wasm m1/f m1/g m2/f
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown -O
	$(NODE) verify-imports.js $(TMPDIR)/bar.wasm m1/f m1/g m2/f
	$(CRABLANGC) bar.rs --target wasm32-unknown-unknown -O -C lto
	$(NODE) verify-imports.js $(TMPDIR)/bar.wasm m1/f m1/g m2/f

	$(CRABLANGC) baz.rs --target wasm32-unknown-unknown
	$(NODE) verify-imports.js $(TMPDIR)/baz.wasm sqlite/allocate sqlite/deallocate

	$(CRABLANGC) log.rs --target wasm32-unknown-unknown
	$(NODE) verify-imports.js $(TMPDIR)/log.wasm test/log
