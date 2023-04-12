include ../tools.mk

# This test ensures that crablangc does not panic with `-o ""` option.

all:
	$(CRABLANGC) -o "" blank.rs 2>&1 | $(CGREP) -i 'panic' && exit 1 || exit 0
