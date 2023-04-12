include ../tools.mk

all:
	$(CRABLANGC) main.rs --emit=mir -o "$(TMPDIR)"/dump.mir

ifdef CRABLANGC_BLESS_TEST
	cp "$(TMPDIR)"/dump.mir dump.mir
else
	$(DIFF) dump.mir "$(TMPDIR)"/dump.mir
endif
