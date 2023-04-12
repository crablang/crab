include ../tools.mk

all:
	$(CRABLANGC) main.rs -o main.rs 2> $(TMPDIR)/file.stderr || echo "failed successfully"
	$(CRABLANGC) main.rs -o . 2> $(TMPDIR)/folder.stderr || echo "failed successfully"

ifdef CRABLANGC_BLESS_TEST
	cp "$(TMPDIR)"/file.stderr file.stderr
	cp "$(TMPDIR)"/folder.stderr folder.stderr
else
	$(DIFF) file.stderr "$(TMPDIR)"/file.stderr
	$(DIFF) folder.stderr "$(TMPDIR)"/folder.stderr
endif
