include ../tools.mk

# ignore-freebsd

all:
	$(CRABLANGC) --emit dep-info main.rs
	$(CGREP) "input.txt" "input.bin" "input.md" < $(TMPDIR)/main.d
