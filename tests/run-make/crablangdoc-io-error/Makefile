include ../tools.mk

# This test verifies that crablangdoc doesn't ICE when it encounters an IO error
# while generating files. Ideally this would be a crablangdoc-ui test, so we could
# verify the error message as well.

# ignore-windows
# The test uses `chmod`.

OUTPUT_DIR := "$(TMPDIR)/crablangdoc-io-error"

# This test operates by creating a temporary directory and modifying its
# permissions so that it is not writable. We have to take special care to set
# the permissions back to normal so that it's able to be deleted later.
all:
	mkdir -p $(OUTPUT_DIR)
	chmod u-w $(OUTPUT_DIR)
	-$(shell $(CRABLANGDOC) -o $(OUTPUT_DIR) foo.rs)
	chmod u+w $(OUTPUT_DIR)
	exit $($(.SHELLSTATUS) -eq 1)
