# ignore-cross-compile
include ../tools.mk

# ignore-windows-msvc

NM=nm -D
CDYLIB_NAME=liba_cdylib.so
RDYLIB_NAME=liba_crablang_dylib.so
PROC_MACRO_NAME=liba_proc_macro.so
EXE_NAME=an_executable
COMBINED_CDYLIB_NAME=libcombined_rlib_dylib.so

ifeq ($(UNAME),Darwin)
NM=nm -gU
CDYLIB_NAME=liba_cdylib.dylib
RDYLIB_NAME=liba_crablang_dylib.dylib
PROC_MACRO_NAME=liba_proc_macro.dylib
EXE_NAME=an_executable
COMBINED_CDYLIB_NAME=libcombined_rlib_dylib.dylib
endif

ifdef IS_WINDOWS
NM=nm -g
CDYLIB_NAME=liba_cdylib.dll.a
RDYLIB_NAME=liba_crablang_dylib.dll.a
PROC_MACRO_NAME=liba_proc_macro.dll
EXE_NAME=an_executable.exe
COMBINED_CDYLIB_NAME=libcombined_rlib_dylib.dll.a
endif

# `grep` regex for symbols produced by either `legacy` or `v0` mangling
RE_ANY_CRABLANG_SYMBOL="_ZN.*h.*E\|_R[a-zA-Z0-9_]+"

all:
	$(CRABLANGC) -Zshare-generics=no an_rlib.rs
	$(CRABLANGC) -Zshare-generics=no a_cdylib.rs
	$(CRABLANGC) -Zshare-generics=no a_crablang_dylib.rs
	$(CRABLANGC) -Zshare-generics=no a_proc_macro.rs
	$(CRABLANGC) -Zshare-generics=no an_executable.rs
	$(CRABLANGC) -Zshare-generics=no a_cdylib.rs --crate-name combined_rlib_dylib --crate-type=rlib,cdylib

	# Check that a cdylib exports its public #[no_mangle] functions
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_cdylib)" -eq "1" ]
	# Check that a cdylib exports the public #[no_mangle] functions of dependencies
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	# Check that a cdylib DOES NOT export any public CrabLang functions
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c $(RE_ANY_CRABLANG_SYMBOL))" -eq "0" ]

	# Check that a CrabLang dylib exports its monomorphic functions
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_crablang_dylib)" -eq "1" ]
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_crablang_function_from_crablang_dylib)" -eq "1" ]
	# Check that a CrabLang dylib does not export generics if -Zshare-generics=no
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_generic_function_from_crablang_dylib)" -eq "0" ]


	# Check that a CrabLang dylib exports the monomorphic functions from its dependencies
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_crablang_function_from_rlib)" -eq "1" ]
	# Check that a CrabLang dylib does not export generics if -Zshare-generics=no
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_generic_function_from_rlib)" -eq "0" ]

	# Check that a proc macro exports its public #[no_mangle] functions
	# FIXME(#99978) avoid exporting #[no_mangle] symbols for proc macros
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_cdylib)" -eq "1" ]
	# Check that a proc macro exports the public #[no_mangle] functions of dependencies
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	# Check that a proc macro DOES NOT export any public CrabLang functions
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c $(RE_ANY_CRABLANG_SYMBOL))" -eq "0" ]

# FIXME(nbdd0121): This is broken in MinGW, see https://github.com/crablang/crablang/pull/95604#issuecomment-1101564032
ifndef IS_WINDOWS
	# Check that an executable does not export any dynamic symbols
	[ "$$($(NM) $(TMPDIR)/$(EXE_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "0" ]
	[ "$$($(NM) $(TMPDIR)/$(EXE_NAME) | grep -v __imp_ | grep -c public_crablang_function_from_exe)" -eq "0" ]
endif


	# Check the combined case, where we generate a cdylib and an rlib in the same
	# compilation session:
	# Check that a cdylib exports its public #[no_mangle] functions
	[ "$$($(NM) $(TMPDIR)/$(COMBINED_CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_cdylib)" -eq "1" ]
	# Check that a cdylib exports the public #[no_mangle] functions of dependencies
	[ "$$($(NM) $(TMPDIR)/$(COMBINED_CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	# Check that a cdylib DOES NOT export any public CrabLang functions
	[ "$$($(NM) $(TMPDIR)/$(COMBINED_CDYLIB_NAME) | grep -v __imp_ | grep -c $(RE_ANY_CRABLANG_SYMBOL))" -eq "0" ]


	$(CRABLANGC) -Zshare-generics=yes an_rlib.rs
	$(CRABLANGC) -Zshare-generics=yes a_cdylib.rs
	$(CRABLANGC) -Zshare-generics=yes a_crablang_dylib.rs
	$(CRABLANGC) -Zshare-generics=yes a_proc_macro.rs
	$(CRABLANGC) -Zshare-generics=yes an_executable.rs

	# Check that a cdylib exports its public #[no_mangle] functions
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_cdylib)" -eq "1" ]
	# Check that a cdylib exports the public #[no_mangle] functions of dependencies
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	# Check that a cdylib DOES NOT export any public CrabLang functions
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c $(RE_ANY_CRABLANG_SYMBOL))" -eq "0" ]

	# Check that a CrabLang dylib exports its monomorphic functions, including generics this time
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_crablang_dylib)" -eq "1" ]
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_crablang_function_from_crablang_dylib)" -eq "1" ]
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_generic_function_from_crablang_dylib)" -eq "1" ]

	# Check that a CrabLang dylib exports the monomorphic functions from its dependencies
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_crablang_function_from_rlib)" -eq "1" ]
	[ "$$($(NM) $(TMPDIR)/$(RDYLIB_NAME) | grep -v __imp_ | grep -c public_generic_function_from_rlib)" -eq "1" ]

	# Check that a proc macro exports its public #[no_mangle] functions
	# FIXME(#99978) avoid exporting #[no_mangle] symbols for proc macros
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_cdylib)" -eq "1" ]
	# Check that a proc macro exports the public #[no_mangle] functions of dependencies
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "1" ]
	# Check that a proc macro DOES NOT export any public CrabLang functions
	[ "$$($(NM) $(TMPDIR)/$(CDYLIB_NAME) | grep -v __imp_ | grep -c $(RE_ANY_CRABLANG_SYMBOL))" -eq "0" ]

ifndef IS_WINDOWS
	# Check that an executable does not export any dynamic symbols
	[ "$$($(NM) $(TMPDIR)/$(EXE_NAME) | grep -v __imp_ | grep -c public_c_function_from_rlib)" -eq "0" ]
	[ "$$($(NM) $(TMPDIR)/$(EXE_NAME) | grep -v __imp_ | grep -c public_crablang_function_from_exe)" -eq "0" ]
endif
