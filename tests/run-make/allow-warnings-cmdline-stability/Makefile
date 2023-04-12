# ignore-cross-compile
include ../tools.mk

# Test that -A warnings makes the 'empty trait list for derive' warning go away
DEP=$(shell $(CRABLANGC) bar.rs)
OUT=$(shell $(CRABLANGC) foo.rs -A warnings 2>&1 | grep "warning" )

all: foo bar
	test -z '$(OUT)'

# These are just to ensure that the above commands actually work
bar:
	$(CRABLANGC) bar.rs

foo: bar
	$(CRABLANGC) foo.rs -A warnings
