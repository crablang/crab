include ../tools.mk

all:
	$(CRABLANGC) --edition=2021 -Dwarnings --crate-type=rlib ../../../library/alloc/src/lib.rs --cfg no_rc
