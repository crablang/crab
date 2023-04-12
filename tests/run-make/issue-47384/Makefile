include ../tools.mk

# only-linux
# ignore-cross-compile

all: main.rs
	$(CRABLANGC) --crate-type lib lib.rs
	$(CRABLANGC) --crate-type cdylib -Clink-args="-Tlinker.ld" main.rs
	# Ensure `#[used]` and `KEEP`-ed section is there
	objdump -s -j".static" $(TMPDIR)/libmain.so
	# Ensure `#[no_mangle]` symbol is there
	nm $(TMPDIR)/libmain.so | $(CGREP) bar
