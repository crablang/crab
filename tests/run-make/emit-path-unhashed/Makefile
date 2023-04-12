include ../tools.mk

OUT=$(TMPDIR)/emit

# --emit KIND=PATH should not affect crate hash vs --emit KIND
all: $(OUT)/a/libfoo.rlib $(OUT)/b/libfoo.rlib $(OUT)/c/libfoo.rlib \
		$(TMPDIR)/libfoo.rlib
	$(CRABLANGC) -Zls $(TMPDIR)/libfoo.rlib > $(TMPDIR)/base.txt
	$(CRABLANGC) -Zls $(OUT)/a/libfoo.rlib > $(TMPDIR)/a.txt
	$(CRABLANGC) -Zls $(OUT)/b/libfoo.rlib > $(TMPDIR)/b.txt
	$(CRABLANGC) -Zls $(OUT)/c/libfoo.rlib > $(TMPDIR)/c.txt

	diff $(TMPDIR)/base.txt $(TMPDIR)/a.txt
	diff $(TMPDIR)/base.txt $(TMPDIR)/b.txt

	# Different KIND parameters do affect hash.
	# diff exits 1 on difference, 2 on trouble
	diff $(TMPDIR)/base.txt $(TMPDIR)/c.txt ; test "$$?" -eq 1

# Default output name
$(TMPDIR)/libfoo.rlib: foo.rs
	$(CRABLANGC) --emit link foo.rs

# Output named with -o
$(OUT)/a/libfoo.rlib: foo.rs
	mkdir -p $(OUT)/a
	$(CRABLANGC) --emit link -o $@ foo.rs

# Output named with KIND=PATH
$(OUT)/b/libfoo.rlib: foo.rs
	mkdir -p $(OUT)/b
	$(CRABLANGC) --emit link=$@ foo.rs

# Output multiple kinds
$(OUT)/c/libfoo.rlib: foo.rs
	mkdir -p $(OUT)/c
	$(CRABLANGC) --emit link=$@,metadata foo.rs
