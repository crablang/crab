# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs -Copt-level=z 2>&1
	$(call RUN,foo)
