include ../tools.mk

OUT=$(TMPDIR)/emit

all: asm llvm-bc llvm-ir obj metadata link dep-info mir

asm: $(OUT)
	$(CRABLANGC) --emit asm=$(OUT)/libfoo.s foo.rs
	test -f $(OUT)/libfoo.s
llvm-bc: $(OUT)
	$(CRABLANGC) --emit llvm-bc=$(OUT)/libfoo.bc foo.rs
	test -f $(OUT)/libfoo.bc
llvm-ir: $(OUT)
	$(CRABLANGC) --emit llvm-ir=$(OUT)/libfoo.ll foo.rs
	test -f $(OUT)/libfoo.ll
obj: $(OUT)
	$(CRABLANGC) --emit obj=$(OUT)/libfoo.o foo.rs
	test -f $(OUT)/libfoo.o
metadata: $(OUT)
	$(CRABLANGC) --emit metadata=$(OUT)/libfoo.rmeta foo.rs
	test -f $(OUT)/libfoo.rmeta
link: $(OUT)
	$(CRABLANGC) --emit link=$(OUT)/libfoo.rlib foo.rs
	test -f $(OUT)/libfoo.rlib
dep-info: $(OUT)
	$(CRABLANGC) --emit dep-info=$(OUT)/libfoo.d foo.rs
	test -f $(OUT)/libfoo.d
mir: $(OUT)
	$(CRABLANGC) --emit mir=$(OUT)/libfoo.mir foo.rs
	test -f $(OUT)/libfoo.mir

$(OUT):
	mkdir -p $(OUT)
