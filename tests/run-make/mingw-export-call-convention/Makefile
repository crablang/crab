include ../tools.mk

# only-windows-gnu

all:
	$(CRABLANGC) foo.rs
	# FIXME: we should make sure __stdcall calling convention is used here
	# but that only works with LLD right now
	nm -g "$(call IMPLIB,foo)" | $(CGREP) bar
