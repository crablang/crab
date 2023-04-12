# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) --crate-type=staticlib nonclike.rs
	$(CC) test.c $(call STATICLIB,nonclike) $(call OUT_EXE,test) \
		$(EXTRACFLAGS) $(EXTRACXXFLAGS)
	$(call RUN,test)
