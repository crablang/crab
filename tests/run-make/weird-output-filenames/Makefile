include ../tools.mk

all:
	cp foo.rs $(TMPDIR)/.foo.rs
	$(CRABLANGC) $(TMPDIR)/.foo.rs 2>&1 \
		| $(CGREP) -e "invalid character.*in crate name:"
	cp foo.rs $(TMPDIR)/.foo.bar
	$(CRABLANGC) $(TMPDIR)/.foo.bar 2>&1 \
		| $(CGREP) -e "invalid character.*in crate name:"
	cp foo.rs $(TMPDIR)/+foo+bar.rs
	$(CRABLANGC) $(TMPDIR)/+foo+bar.rs 2>&1 \
		| $(CGREP) -e "invalid character.*in crate name:"
	cp foo.rs $(TMPDIR)/-foo.rs
	$(CRABLANGC) $(TMPDIR)/-foo.rs 2>&1 \
		| $(CGREP) 'crate names cannot start with a `-`'
