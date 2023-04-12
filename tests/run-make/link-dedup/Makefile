# ignore-msvc

include ../tools.mk

all:
	$(CRABLANGC) depa.rs
	$(CRABLANGC) depb.rs
	$(CRABLANGC) depc.rs
	$(CRABLANGC) empty.rs --cfg bar 2>&1 | $(CGREP) '"-ltesta" "-ltestb" "-ltesta"'
	$(CRABLANGC) empty.rs 2>&1 | $(CGREP) '"-ltesta"'
	$(CRABLANGC) empty.rs 2>&1 | $(CGREP) -v '"-ltestb"'
	$(CRABLANGC) empty.rs 2>&1 | $(CGREP) -v '"-ltesta" "-ltesta" "-ltesta"'
