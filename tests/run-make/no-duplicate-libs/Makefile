# ignore-cross-compile
include ../tools.mk

ifdef IS_MSVC
# FIXME(#27979)
all:
else
all: $(call STATICLIB,foo) $(call STATICLIB,bar)
	$(CRABLANGC) main.rs
	$(call RUN,main)
endif
