# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) checkcrablang.rs
	$(CC) test.c $(call STATICLIB,checkcrablang) $(call OUT_EXE,test) $(EXTRACFLAGS)
	$(call RUN,test)
