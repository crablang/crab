# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) -Copt-level=0 --emit=llvm-bc,llvm-ir,asm,obj,link test-24876.rs
	$(CRABLANGC) -Copt-level=1 --emit=llvm-bc,llvm-ir,asm,obj,link test-24876.rs
	$(CRABLANGC) -Copt-level=2 --emit=llvm-bc,llvm-ir,asm,obj,link test-24876.rs
	$(CRABLANGC) -Copt-level=3 --emit=llvm-bc,llvm-ir,asm,obj,link test-24876.rs
	$(CRABLANGC) -Copt-level=s --emit=llvm-bc,llvm-ir,asm,obj,link test-24876.rs
	$(CRABLANGC) -Copt-level=z --emit=llvm-bc,llvm-ir,asm,obj,link test-24876.rs
	$(CRABLANGC) -Copt-level=0 --emit=llvm-bc,llvm-ir,asm,obj,link test-26235.rs
	$(call RUN,test-26235) || exit 1
	$(CRABLANGC) -Copt-level=1 --emit=llvm-bc,llvm-ir,asm,obj,link test-26235.rs
	$(call RUN,test-26235) || exit 1
	$(CRABLANGC) -Copt-level=2 --emit=llvm-bc,llvm-ir,asm,obj,link test-26235.rs
	$(call RUN,test-26235) || exit 1
	$(CRABLANGC) -Copt-level=3 --emit=llvm-bc,llvm-ir,asm,obj,link test-26235.rs
	$(call RUN,test-26235) || exit 1
	$(CRABLANGC) -Copt-level=s --emit=llvm-bc,llvm-ir,asm,obj,link test-26235.rs
	$(call RUN,test-26235) || exit 1
	$(CRABLANGC) -Copt-level=z --emit=llvm-bc,llvm-ir,asm,obj,link test-26235.rs
	$(call RUN,test-26235) || exit 1
