# ignore-cross-compile
include ../tools.mk

# Test that -A warnings makes the 'empty trait list for derive' warning go away
OUT=$(shell $(CRABLANGC) foo.rs -A warnings 2>&1 | grep "warning" )

all: foo
	test -z '$(OUT)'

# This is just to make sure the above command actually succeeds
foo:
	$(CRABLANGC) foo.rs -A warnings
