include ../tools.mk

all:
	$(CRABLANGC) --print crate-name 2>&1 | diff - no-input-file.stderr
