# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs -g -O
	CRABLANGC="$(CRABLANGC_ORIGINAL)" $(call RUN,foo)
