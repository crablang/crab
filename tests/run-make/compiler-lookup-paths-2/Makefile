include ../tools.mk

all:
	mkdir -p $(TMPDIR)/a $(TMPDIR)/b
	$(CRABLANGC) a.rs && mv $(TMPDIR)/liba.rlib $(TMPDIR)/a
	$(CRABLANGC) b.rs -L $(TMPDIR)/a && mv $(TMPDIR)/libb.rlib $(TMPDIR)/b
	$(CRABLANGC) c.rs -L crate=$(TMPDIR)/b -L dependency=$(TMPDIR)/a \
		&& exit 1 || exit 0
