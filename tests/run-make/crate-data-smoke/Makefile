include ../tools.mk

all:
	[ `$(CRABLANGC) --print crate-name crate.rs` = "foo" ]
	[ `$(CRABLANGC) --print file-names crate.rs` = "$(call BIN,foo)" ]
	[ `$(CRABLANGC) --print file-names --crate-type=lib \
		--test crate.rs` = "$(call BIN,foo)" ]
	[ `$(CRABLANGC) --print file-names --test lib.rs` = "$(call BIN,mylib)" ]
	$(CRABLANGC) --print file-names lib.rs
	$(CRABLANGC) --print file-names rlib.rs
