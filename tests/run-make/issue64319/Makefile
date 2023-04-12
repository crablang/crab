# ignore-cross-compile
include ../tools.mk

# Different optimization levels imply different values for `-Zshare-generics`,
# so try out a whole bunch of combinations to make sure everything is compatible
all:
	# First up, try some defaults
	$(CRABLANGC) --crate-type rlib foo.rs
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=3

	# Next try mixing up some things explicitly
	$(CRABLANGC) --crate-type rlib foo.rs -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -Z share-generics=no
	$(CRABLANGC) --crate-type rlib foo.rs -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -Z share-generics=yes
	$(CRABLANGC) --crate-type rlib foo.rs -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -Z share-generics=no
	$(CRABLANGC) --crate-type rlib foo.rs -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -Z share-generics=yes

	# Now combine a whole bunch of options together
	$(CRABLANGC) --crate-type rlib foo.rs
	$(CRABLANGC) --crate-type dylib bar.rs
	$(CRABLANGC) --crate-type dylib bar.rs -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=1
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=1 -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=1 -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=2
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=2 -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=2 -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=3
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=3 -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=3 -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=s
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=s -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=s -Z share-generics=yes
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=z
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=z -Z share-generics=no
	$(CRABLANGC) --crate-type dylib bar.rs -C opt-level=z -Z share-generics=yes
