include ../tools.mk

# Checks that all the targets returned by `crablangc --print target-list` are valid
# target specifications
all:
	for target in $(shell $(BARE_CRABLANGC) --print target-list); do \
		$(BARE_CRABLANGC) --target $$target --print sysroot; \
	done
