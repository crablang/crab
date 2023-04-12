include ../tools.mk

# ignore-windows-msvc

all:
	$(CRABLANGC) --emit=obj app.rs
	nm $(TMPDIR)/app.o | $(CGREP) crablang_begin_unwind
	nm $(TMPDIR)/app.o | $(CGREP) crablang_eh_personality
	nm $(TMPDIR)/app.o | $(CGREP) __rg_oom
