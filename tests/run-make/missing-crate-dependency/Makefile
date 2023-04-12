include ../tools.mk

all:
	$(CRABLANGC) --crate-type=rlib crateA.rs
	$(CRABLANGC) --crate-type=rlib crateB.rs
	$(call REMOVE_RLIBS,crateA)
	# Ensure crateC fails to compile since dependency crateA is missing
	$(CRABLANGC) crateC.rs 2>&1 | \
		$(CGREP) 'can'"'"'t find crate for `crateA` which `crateB` depends on'
