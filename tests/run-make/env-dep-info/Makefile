include ../tools.mk

# FIXME(eddyb) provide `HOST_RUSTC` and `TARGET_RUSTC`
# instead of hardcoding them everywhere they're needed.
ifeq ($(IS_MUSL_HOST),1)
ADDITIONAL_ARGS := $(RUSTFLAGS)
endif

all:
	EXISTING_ENV=1 EXISTING_OPT_ENV=1 $(RUSTC) --emit dep-info main.rs
	$(CGREP) "# env-dep:EXISTING_ENV=1" < $(TMPDIR)/main.d
	$(CGREP) "# env-dep:EXISTING_OPT_ENV=1" < $(TMPDIR)/main.d
	$(CGREP) "# env-dep:NONEXISTENT_OPT_ENV" < $(TMPDIR)/main.d
	$(CGREP) "# env-dep:ESCAPE\nESCAPE\\" < $(TMPDIR)/main.d
	# Proc macro
	$(BARE_RUSTC) $(ADDITIONAL_ARGS) --out-dir $(TMPDIR) macro_def.rs
	EXISTING_PROC_MACRO_ENV=1 $(RUSTC) --emit dep-info macro_use.rs
	$(CGREP) "# env-dep:EXISTING_PROC_MACRO_ENV=1" < $(TMPDIR)/macro_use.d
	$(CGREP) "# env-dep:NONEXISTENT_PROC_MACEO_ENV" < $(TMPDIR)/macro_use.d
