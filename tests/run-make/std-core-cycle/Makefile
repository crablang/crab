# ignore-cross-compile
include ../tools.mk

ifeq ($(UNAME),Darwin)
FLAGS :=
else
ifdef IS_WINDOWS
FLAGS :=
else
FLAGS := -C link-args=-Wl,--no-undefined
endif
endif

all:
	$(CRABLANGC) bar.rs
	$(CRABLANGC) foo.rs $(FLAGS)
	$(CRABLANGC) foo.rs $(FLAGS) -C panic=abort
