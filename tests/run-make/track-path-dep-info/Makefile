include ../tools.mk

# FIXME(eddyb) provide `HOST_CRABLANGC` and `TARGET_CRABLANGC`
# instead of hardcoding them everywhere they're needed.
ifeq ($(IS_MUSL_HOST),1)
ADDITIONAL_ARGS := $(CRABLANGFLAGS)
endif

all:
	# Proc macro
	$(BARE_CRABLANGC) $(ADDITIONAL_ARGS) --out-dir $(TMPDIR) macro_def.rs
	EXISTING_PROC_MACRO_ENV=1 $(CRABLANGC) --emit dep-info macro_use.rs
	$(CGREP) "emojis.txt:" < $(TMPDIR)/macro_use.d
