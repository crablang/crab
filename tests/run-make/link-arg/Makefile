include ../tools.mk
CRABLANGC_FLAGS = -C link-arg="-lfoo" -C link-arg="-lbar" --print link-args

all:
	$(CRABLANGC) $(CRABLANGC_FLAGS) empty.rs | $(CGREP) lfoo lbar
