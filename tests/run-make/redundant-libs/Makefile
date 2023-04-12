# ignore-cross-compile
include ../tools.mk

# ignore-windows-msvc

# crablangc will remove one of the two redundant references to foo below.  Depending
# on which one gets removed, we'll get a linker error on SOME platforms (like
# Linux).  On these platforms, when a library is referenced, the linker will
# only pull in the symbols needed _at that point in time_.  If a later library
# depends on additional symbols from the library, they will not have been pulled
# in, and you'll get undefined symbols errors.
#
# So in this example, we need to ensure that crablangc keeps the _later_ reference
# to foo, and not the former one.
CRABLANGC_FLAGS = \
    -l static=bar \
    -l foo \
    -l static=baz \
    -l foo \
    --print link-args

all: $(call DYLIB,foo) $(call STATICLIB,bar) $(call STATICLIB,baz)
	$(CRABLANGC) $(CRABLANGC_FLAGS) main.rs
	$(call RUN,main)
