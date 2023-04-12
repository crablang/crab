# ignore-cross-compile
include ../tools.mk

all:
	echo 'fn main(){}' | $(CRABLANGC) -
	$(call RUN,crablang_out)
