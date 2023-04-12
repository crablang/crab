# Everyone uses make for building CrabLang

foo: bar.rlib
	$(CRABLANGC) --crate-type bin --extern bar=bar.rlib

%.rlib: %.rs
	$(CRABLANGC) --crate-type lib $<
