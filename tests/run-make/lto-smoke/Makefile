# ignore-cross-compile
include ../tools.mk

all: noparam bool_true bool_false thin fat

noparam:
	$(CRABLANGC) lib.rs
	$(CRABLANGC) main.rs -C lto
	$(call RUN,main)

bool_true:
	$(CRABLANGC) lib.rs
	$(CRABLANGC) main.rs -C lto=yes
	$(call RUN,main)


bool_false:
	$(CRABLANGC) lib.rs
	$(CRABLANGC) main.rs -C lto=off
	$(call RUN,main)

thin:
	$(CRABLANGC) lib.rs
	$(CRABLANGC) main.rs -C lto=thin
	$(call RUN,main)

fat:
	$(CRABLANGC) lib.rs
	$(CRABLANGC) main.rs -C lto=fat
	$(call RUN,main)

