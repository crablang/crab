include ../tools.mk

# only-windows

all:
	$(CRABLANGC) -o "$(TMPDIR)/hopefullydoesntexist bar.exe" hello.rs
	$(CRABLANGC) spawn.rs
	$(TMPDIR)/spawn.exe
