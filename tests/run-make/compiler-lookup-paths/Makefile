include ../tools.mk

# ignore-wasm32 (need a C compiler)
# ignore-wasm64 (need a C compiler)

all: $(TMPDIR)/libnative.a
	mkdir -p $(TMPDIR)/crate
	mkdir -p $(TMPDIR)/native
	mv $(TMPDIR)/libnative.a $(TMPDIR)/native
	$(CRABLANGC) a.rs
	mv $(TMPDIR)/liba.rlib $(TMPDIR)/crate
	$(CRABLANGC) b.rs -L native=$(TMPDIR)/crate && exit 1 || exit 0
	$(CRABLANGC) b.rs -L dependency=$(TMPDIR)/crate && exit 1 || exit 0
	$(CRABLANGC) b.rs -L crate=$(TMPDIR)/crate
	$(CRABLANGC) b.rs -L all=$(TMPDIR)/crate
	$(CRABLANGC) c.rs -L native=$(TMPDIR)/crate && exit 1 || exit 0
	$(CRABLANGC) c.rs -L crate=$(TMPDIR)/crate && exit 1 || exit 0
	$(CRABLANGC) c.rs -L dependency=$(TMPDIR)/crate
	$(CRABLANGC) c.rs -L all=$(TMPDIR)/crate
	$(CRABLANGC) d.rs -L dependency=$(TMPDIR)/native && exit 1 || exit 0
	$(CRABLANGC) d.rs -L crate=$(TMPDIR)/native && exit 1 || exit 0
	$(CRABLANGC) d.rs -L native=$(TMPDIR)/native
	$(CRABLANGC) d.rs -L all=$(TMPDIR)/native
	# Deduplication tests:
	#   Same hash, no errors.
	mkdir -p $(TMPDIR)/e1
	mkdir -p $(TMPDIR)/e2
	$(CRABLANGC) e.rs -o $(TMPDIR)/e1/libe.rlib
	$(CRABLANGC) e.rs -o $(TMPDIR)/e2/libe.rlib
	$(CRABLANGC) f.rs -L $(TMPDIR)/e1 -L $(TMPDIR)/e2
	$(CRABLANGC) f.rs -L crate=$(TMPDIR)/e1 -L $(TMPDIR)/e2
	$(CRABLANGC) f.rs -L crate=$(TMPDIR)/e1 -L crate=$(TMPDIR)/e2
	#   Different hash, errors.
	$(CRABLANGC) e2.rs -o $(TMPDIR)/e2/libe.rlib
	$(CRABLANGC) f.rs -L $(TMPDIR)/e1 -L $(TMPDIR)/e2 && exit 1 || exit 0
	$(CRABLANGC) f.rs -L crate=$(TMPDIR)/e1 -L $(TMPDIR)/e2 && exit 1 || exit 0
	$(CRABLANGC) f.rs -L crate=$(TMPDIR)/e1 -L crate=$(TMPDIR)/e2 && exit 1 || exit 0
	#   Native/dependency paths don't cause errors.
	$(CRABLANGC) f.rs -L native=$(TMPDIR)/e1 -L $(TMPDIR)/e2
	$(CRABLANGC) f.rs -L dependency=$(TMPDIR)/e1 -L $(TMPDIR)/e2
	$(CRABLANGC) f.rs -L dependency=$(TMPDIR)/e1 -L crate=$(TMPDIR)/e2
