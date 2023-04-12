include ../../run-make/tools.mk

# This example shows how to implement a crablangc driver that retrieves MIR bodies
# together with the borrow checker information.

# How to run this
# $ ./x.py test tests/run-make-fulldeps/obtain-borrowck

DRIVER_BINARY := "$(TMPDIR)"/driver
SYSROOT := $(shell $(CRABLANGC) --print sysroot)

ifdef IS_WINDOWS
LIBSTD := -L "$(SYSROOT)\\lib\\crablanglib\\$(TARGET)\\lib"
else
LIBSTD :=
endif

all:
	$(CRABLANGC) driver.rs -o "$(DRIVER_BINARY)"
	$(TARGET_RPATH_ENV) "$(DRIVER_BINARY)" --sysroot $(SYSROOT) $(LIBSTD) test.rs -o "$(TMPDIR)/driver_test" > "$(TMPDIR)"/output.stdout

ifdef CRABLANGC_BLESS_TEST
	cp "$(TMPDIR)"/output.stdout output.stdout
else
	$(DIFF) output.stdout "$(TMPDIR)"/output.stdout
endif
