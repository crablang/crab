include ../tools.mk

all:
	$(CRABLANGC) panic.rs
	$(CRABLANGC) -C panic=abort --emit=obj app.rs -L $(TMPDIR)
