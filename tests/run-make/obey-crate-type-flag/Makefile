# ignore-cross-compile
include ../tools.mk

# check that crablangc builds all crate_type attributes
# delete rlib
# delete whatever dylib is made for this system
# check that crablangc only builds --crate-type flags, ignoring attributes
# fail if an rlib was built
all:
	$(CRABLANGC) test.rs
	$(call REMOVE_RLIBS,test)
	$(call REMOVE_DYLIBS,test)
	$(CRABLANGC) --crate-type dylib test.rs
	$(call REMOVE_RLIBS,test) && exit 1 || exit 0
