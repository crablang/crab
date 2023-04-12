# ignore-cross-compile
include ../tools.mk

all: $(call DYLIB,return1) $(call DYLIB,return2) $(call NATIVE_STATICLIB,return3)
	ls $(TMPDIR)
	$(BARE_CRABLANGC) --print cfg --target x86_64-unknown-linux-musl | $(CGREP) crt-static

	$(CRABLANGC) no-deps.rs --cfg foo
	$(call RUN,no-deps)
	$(CRABLANGC) no-deps.rs --cfg bar
	$(call RUN,no-deps)

	$(CRABLANGC) dep.rs
	$(CRABLANGC) with-deps.rs --cfg foo
	$(call RUN,with-deps)
	$(CRABLANGC) with-deps.rs --cfg bar
	$(call RUN,with-deps)

	$(CRABLANGC) dep-with-staticlib.rs
	$(CRABLANGC) with-staticlib-deps.rs --cfg foo
	$(call RUN,with-staticlib-deps)
	$(CRABLANGC) with-staticlib-deps.rs --cfg bar
	$(call RUN,with-staticlib-deps)
