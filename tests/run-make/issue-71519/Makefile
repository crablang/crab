include ../tools.mk

# ignore-msvc
# needs-crablang-lld
# ignore-s390x lld does not yet support s390x as target
all:
	CRABLANGC_LOG=crablangc_codegen_ssa::back::link=info $(CRABLANGC) -Z gcc-ld=lld -C link-args=-Wl,-v main.rs 2> $(TMPDIR)/output.txt
	$(CGREP) -e "^LLD [0-9]+\.[0-9]+\.[0-9]+" < $(TMPDIR)/output.txt
