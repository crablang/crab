include ../tools.mk

# ignore-windows-msvc

all:
	$(CRABLANGC) -C opt-level=3 --emit=obj used.rs
	nm $(TMPDIR)/used.o | $(CGREP) FOO
