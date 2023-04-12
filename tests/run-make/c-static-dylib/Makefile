# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,cfoo)
	$(CRABLANGC) foo.rs -C prefer-dynamic
	$(CRABLANGC) bar.rs
	rm $(call NATIVE_STATICLIB,cfoo)
	$(call RUN,bar)
	$(call REMOVE_DYLIBS,foo)
	$(call FAIL,bar)
