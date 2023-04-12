# needs-llvm-components: x86 arm

include ../tools.mk

all: default
	$(CRABLANGC) --target x86_64-pc-windows-gnu --print cfg | $(CGREP) windows
	$(CRABLANGC) --target x86_64-pc-windows-gnu --print cfg | $(CGREP) x86_64
	$(CRABLANGC) --target i686-pc-windows-msvc --print cfg | $(CGREP) msvc
	$(CRABLANGC) --target i686-apple-darwin --print cfg | $(CGREP) macos
	$(CRABLANGC) --target i686-unknown-linux-gnu --print cfg | $(CGREP) gnu
	$(CRABLANGC) --target arm-unknown-linux-gnueabihf --print cfg | $(CGREP) target_abi=
	$(CRABLANGC) --target arm-unknown-linux-gnueabihf --print cfg | $(CGREP) eabihf

ifdef IS_WINDOWS
default:
	$(CRABLANGC) --print cfg | $(CGREP) windows
else
default:
	$(CRABLANGC) --print cfg | $(CGREP) unix
endif
