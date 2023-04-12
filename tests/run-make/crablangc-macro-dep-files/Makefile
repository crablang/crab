include ../tools.mk

# FIXME(eddyb) provide `HOST_CRABLANGC` and `TARGET_CRABLANGC`
# instead of hardcoding them everywhere they're needed.
ifeq ($(IS_MUSL_HOST),1)
ADDITIONAL_ARGS := $(CRABLANGFLAGS)
endif
all:
	$(BARE_CRABLANGC) $(ADDITIONAL_ARGS) foo.rs --out-dir $(TMPDIR)
	$(CRABLANGC) bar.rs --target $(TARGET) --emit dep-info
	$(CGREP) -v "proc-macro source" < $(TMPDIR)/bar.d
