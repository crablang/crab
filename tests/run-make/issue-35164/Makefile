include ../tools.mk

all:
	$(CRABLANGC) main.rs --error-format json 2>&1 | $(CGREP) -e '"byte_start":23\b' '"byte_end":29\b'
