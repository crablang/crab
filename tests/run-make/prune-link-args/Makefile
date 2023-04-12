# ignore-cross-compile
include ../tools.mk

# ignore-windows

# Notice the space in the end, this emulates the output of pkg-config
CRABLANGC_FLAGS = -C link-args="-lc "

all:
	$(CRABLANGC) $(CRABLANGC_FLAGS) empty.rs
