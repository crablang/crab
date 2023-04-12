#include "llvm-c/BitReader.h"
#include "llvm-c/Core.h"
#include "llvm-c/Object.h"
#include "llvm/ADT/ArrayRef.h"
#include "llvm/ADT/DenseSet.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/Analysis/Lint.h"
#include "llvm/Analysis/Passes.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/InlineAsm.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Support/Debug.h"
#include "llvm/Support/DynamicLibrary.h"
#include "llvm/Support/FormattedStream.h"
#include "llvm/Support/JSON.h"
#include "llvm/Support/Host.h"
#include "llvm/Support/Memory.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/Timer.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Target/TargetMachine.h"
#include "llvm/Target/TargetOptions.h"
#include "llvm/Transforms/IPO.h"
#include "llvm/Transforms/Instrumentation.h"
#include "llvm/Transforms/Scalar.h"
#include "llvm/Transforms/Vectorize.h"

#define LLVM_VERSION_GE(major, minor)                                          \
  (LLVM_VERSION_MAJOR > (major) ||                                             \
   LLVM_VERSION_MAJOR == (major) && LLVM_VERSION_MINOR >= (minor))

#define LLVM_VERSION_LT(major, minor) (!LLVM_VERSION_GE((major), (minor)))

#include "llvm/IR/LegacyPassManager.h"

#include "llvm/Bitcode/BitcodeReader.h"
#include "llvm/Bitcode/BitcodeWriter.h"

#include "llvm/IR/DIBuilder.h"
#include "llvm/IR/DebugInfo.h"
#include "llvm/IR/IRPrintingPasses.h"
#include "llvm/Linker/Linker.h"

#if LLVM_VERSION_GE(16, 0)
#include "llvm/TargetParser/Triple.h"
#else
#include "llvm/ADT/Triple.h"
#endif

extern "C" void LLVMCrabLangSetLastError(const char *);

enum class LLVMCrabLangResult { Success, Failure };

enum LLVMCrabLangAttribute {
  AlwaysInline = 0,
  ByVal = 1,
  Cold = 2,
  InlineHint = 3,
  MinSize = 4,
  Naked = 5,
  NoAlias = 6,
  NoCapture = 7,
  NoInline = 8,
  NonNull = 9,
  NoRedZone = 10,
  NoReturn = 11,
  NoUnwind = 12,
  OptimizeForSize = 13,
  ReadOnly = 14,
  SExt = 15,
  StructRet = 16,
  UWTable = 17,
  ZExt = 18,
  InReg = 19,
  SanitizeThread = 20,
  SanitizeAddress = 21,
  SanitizeMemory = 22,
  NonLazyBind = 23,
  OptimizeNone = 24,
  ReturnsTwice = 25,
  ReadNone = 26,
  SanitizeHWAddress = 28,
  WillReturn = 29,
  StackProtectReq = 30,
  StackProtectStrong = 31,
  StackProtect = 32,
  NoUndef = 33,
  SanitizeMemTag = 34,
  NoCfCheck = 35,
  ShadowCallStack = 36,
  AllocSize = 37,
#if LLVM_VERSION_GE(15, 0)
  AllocatedPointer = 38,
  AllocAlign = 39,
#endif
};

typedef struct OpaqueCrabLangString *CrabLangStringRef;
typedef struct LLVMOpaqueTwine *LLVMTwineRef;
typedef struct LLVMOpaqueSMDiagnostic *LLVMSMDiagnosticRef;

extern "C" void LLVMCrabLangStringWriteImpl(CrabLangStringRef Str, const char *Ptr,
                                        size_t Size);

class RawCrabLangStringOstream : public llvm::raw_ostream {
  CrabLangStringRef Str;
  uint64_t Pos;

  void write_impl(const char *Ptr, size_t Size) override {
    LLVMCrabLangStringWriteImpl(Str, Ptr, Size);
    Pos += Size;
  }

  uint64_t current_pos() const override { return Pos; }

public:
  explicit RawCrabLangStringOstream(CrabLangStringRef Str) : Str(Str), Pos(0) {}

  ~RawCrabLangStringOstream() {
    // LLVM requires this.
    flush();
  }
};
