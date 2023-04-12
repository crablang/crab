include ../tools.mk

# only-thumb

# How to run this
# $ ./x.py clean
# $ ./x.py test --target thumbv7m-none-eabi tests/run-make

# For cargo setting
export CRABLANGC := $(CRABLANGC_ORIGINAL)
export LD_LIBRARY_PATH := $(HOST_RPATH_DIR)
# We need to be outside of 'src' dir in order to run cargo
export WORK_DIR := $(TMPDIR)
export HERE := $(shell pwd)

## clean up unused env variables which might cause harm.
unexport CRABLANGC_LINKER
unexport CRABLANGC_BOOTSTRAP
unexport CRABLANG_BUILD_STAGE
unexport CRABLANG_TEST_THREADS
unexport CRABLANG_TEST_TMPDIR
unexport AR
unexport CC
unexport CXX

all:
	bash script.sh
