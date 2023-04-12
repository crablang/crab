#include "LLVMWrapper.h"
#include "llvm/IR/DebugInfoMetadata.h"
#include "llvm/IR/DiagnosticHandler.h"
#include "llvm/IR/DiagnosticInfo.h"
#include "llvm/IR/DiagnosticPrinter.h"
#include "llvm/IR/GlobalVariable.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Intrinsics.h"
#include "llvm/IR/IntrinsicsARM.h"
#include "llvm/IR/Mangler.h"
#if LLVM_VERSION_GE(16, 0)
#include "llvm/Support/ModRef.h"
#endif
#include "llvm/Object/Archive.h"
#include "llvm/Object/COFFImportFile.h"
#include "llvm/Object/ObjectFile.h"
#include "llvm/Pass.h"
#include "llvm/Bitcode/BitcodeWriter.h"
#include "llvm/Support/Signals.h"
#if LLVM_VERSION_LT(16, 0)
#include "llvm/ADT/Optional.h"
#endif

#include <iostream>

//===----------------------------------------------------------------------===
//
// This file defines alternate interfaces to core functions that are more
// readily callable by CrabLang's FFI.
//
//===----------------------------------------------------------------------===

using namespace llvm;
using namespace llvm::sys;
using namespace llvm::object;

// LLVMAtomicOrdering is already an enum - don't create another
// one.
static AtomicOrdering fromCrabLang(LLVMAtomicOrdering Ordering) {
  switch (Ordering) {
  case LLVMAtomicOrderingNotAtomic:
    return AtomicOrdering::NotAtomic;
  case LLVMAtomicOrderingUnordered:
    return AtomicOrdering::Unordered;
  case LLVMAtomicOrderingMonotonic:
    return AtomicOrdering::Monotonic;
  case LLVMAtomicOrderingAcquire:
    return AtomicOrdering::Acquire;
  case LLVMAtomicOrderingRelease:
    return AtomicOrdering::Release;
  case LLVMAtomicOrderingAcquireRelease:
    return AtomicOrdering::AcquireRelease;
  case LLVMAtomicOrderingSequentiallyConsistent:
    return AtomicOrdering::SequentiallyConsistent;
  }

  report_fatal_error("Invalid LLVMAtomicOrdering value!");
}

static LLVM_THREAD_LOCAL char *LastError;

// Custom error handler for fatal LLVM errors.
//
// Notably it exits the process with code 101, unlike LLVM's default of 1.
static void FatalErrorHandler(void *UserData,
                              const char* Reason,
                              bool GenCrashDiag) {
  // Do the same thing that the default error handler does.
  std::cerr << "LLVM ERROR: " << Reason << std::endl;

  // Since this error handler exits the process, we have to run any cleanup that
  // LLVM would run after handling the error. This might change with an LLVM
  // upgrade.
  sys::RunInterruptHandlers();

  exit(101);
}

extern "C" void LLVMCrabLangInstallFatalErrorHandler() {
  install_fatal_error_handler(FatalErrorHandler);
}

extern "C" void LLVMCrabLangDisableSystemDialogsOnCrash() {
  sys::DisableSystemDialogsOnCrash();
}

extern "C" char *LLVMCrabLangGetLastError(void) {
  char *Ret = LastError;
  LastError = nullptr;
  return Ret;
}

extern "C" void LLVMCrabLangSetLastError(const char *Err) {
  free((void *)LastError);
  LastError = strdup(Err);
}

extern "C" LLVMContextRef LLVMCrabLangContextCreate(bool shouldDiscardNames) {
  auto ctx = new LLVMContext();
  ctx->setDiscardValueNames(shouldDiscardNames);
  return wrap(ctx);
}

extern "C" void LLVMCrabLangSetNormalizedTarget(LLVMModuleRef M,
                                            const char *Triple) {
  unwrap(M)->setTargetTriple(Triple::normalize(Triple));
}

extern "C" void LLVMCrabLangPrintPassTimings() {
  raw_fd_ostream OS(2, false); // stderr.
  TimerGroup::printAll(OS);
}

extern "C" LLVMValueRef LLVMCrabLangGetNamedValue(LLVMModuleRef M, const char *Name,
                                              size_t NameLen) {
  return wrap(unwrap(M)->getNamedValue(StringRef(Name, NameLen)));
}

extern "C" LLVMValueRef LLVMCrabLangGetOrInsertFunction(LLVMModuleRef M,
                                                    const char *Name,
                                                    size_t NameLen,
                                                    LLVMTypeRef FunctionTy) {
  return wrap(unwrap(M)
                  ->getOrInsertFunction(StringRef(Name, NameLen),
                                        unwrap<FunctionType>(FunctionTy))
                  .getCallee()
  );
}

extern "C" LLVMValueRef
LLVMCrabLangGetOrInsertGlobal(LLVMModuleRef M, const char *Name, size_t NameLen, LLVMTypeRef Ty) {
  Module *Mod = unwrap(M);
  StringRef NameRef(Name, NameLen);

  // We don't use Module::getOrInsertGlobal because that returns a Constant*,
  // which may either be the real GlobalVariable*, or a constant bitcast of it
  // if our type doesn't match the original declaration. We always want the
  // GlobalVariable* so we can access linkage, visibility, etc.
  GlobalVariable *GV = Mod->getGlobalVariable(NameRef, true);
  if (!GV)
    GV = new GlobalVariable(*Mod, unwrap(Ty), false,
                            GlobalValue::ExternalLinkage, nullptr, NameRef);
  return wrap(GV);
}

extern "C" LLVMValueRef
LLVMCrabLangInsertPrivateGlobal(LLVMModuleRef M, LLVMTypeRef Ty) {
  return wrap(new GlobalVariable(*unwrap(M),
                                 unwrap(Ty),
                                 false,
                                 GlobalValue::PrivateLinkage,
                                 nullptr));
}

static Attribute::AttrKind fromCrabLang(LLVMCrabLangAttribute Kind) {
  switch (Kind) {
  case AlwaysInline:
    return Attribute::AlwaysInline;
  case ByVal:
    return Attribute::ByVal;
  case Cold:
    return Attribute::Cold;
  case InlineHint:
    return Attribute::InlineHint;
  case MinSize:
    return Attribute::MinSize;
  case Naked:
    return Attribute::Naked;
  case NoAlias:
    return Attribute::NoAlias;
  case NoCapture:
    return Attribute::NoCapture;
  case NoCfCheck:
    return Attribute::NoCfCheck;
  case NoInline:
    return Attribute::NoInline;
  case NonNull:
    return Attribute::NonNull;
  case NoRedZone:
    return Attribute::NoRedZone;
  case NoReturn:
    return Attribute::NoReturn;
  case NoUnwind:
    return Attribute::NoUnwind;
  case OptimizeForSize:
    return Attribute::OptimizeForSize;
  case ReadOnly:
    return Attribute::ReadOnly;
  case SExt:
    return Attribute::SExt;
  case StructRet:
    return Attribute::StructRet;
  case UWTable:
    return Attribute::UWTable;
  case ZExt:
    return Attribute::ZExt;
  case InReg:
    return Attribute::InReg;
  case SanitizeThread:
    return Attribute::SanitizeThread;
  case SanitizeAddress:
    return Attribute::SanitizeAddress;
  case SanitizeMemory:
    return Attribute::SanitizeMemory;
  case NonLazyBind:
    return Attribute::NonLazyBind;
  case OptimizeNone:
    return Attribute::OptimizeNone;
  case ReturnsTwice:
    return Attribute::ReturnsTwice;
  case ReadNone:
    return Attribute::ReadNone;
  case SanitizeHWAddress:
    return Attribute::SanitizeHWAddress;
  case WillReturn:
    return Attribute::WillReturn;
  case StackProtectReq:
    return Attribute::StackProtectReq;
  case StackProtectStrong:
    return Attribute::StackProtectStrong;
  case StackProtect:
    return Attribute::StackProtect;
  case NoUndef:
    return Attribute::NoUndef;
  case SanitizeMemTag:
    return Attribute::SanitizeMemTag;
  case ShadowCallStack:
    return Attribute::ShadowCallStack;
  case AllocSize:
    return Attribute::AllocSize;
#if LLVM_VERSION_GE(15, 0)
  case AllocatedPointer:
    return Attribute::AllocatedPointer;
  case AllocAlign:
    return Attribute::AllocAlign;
#endif
  }
  report_fatal_error("bad AttributeKind");
}

template<typename T> static inline void AddAttributes(T *t, unsigned Index,
                                                      LLVMAttributeRef *Attrs, size_t AttrsLen) {
  AttributeList PAL = t->getAttributes();
  AttrBuilder B(t->getContext());
  for (LLVMAttributeRef Attr : ArrayRef<LLVMAttributeRef>(Attrs, AttrsLen))
    B.addAttribute(unwrap(Attr));
  AttributeList PALNew = PAL.addAttributesAtIndex(t->getContext(), Index, B);
  t->setAttributes(PALNew);
}

extern "C" void LLVMCrabLangAddFunctionAttributes(LLVMValueRef Fn, unsigned Index,
                                              LLVMAttributeRef *Attrs, size_t AttrsLen) {
  Function *F = unwrap<Function>(Fn);
  AddAttributes(F, Index, Attrs, AttrsLen);
}

extern "C" void LLVMCrabLangAddCallSiteAttributes(LLVMValueRef Instr, unsigned Index,
                                              LLVMAttributeRef *Attrs, size_t AttrsLen) {
  CallBase *Call = unwrap<CallBase>(Instr);
  AddAttributes(Call, Index, Attrs, AttrsLen);
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateAttrNoValue(LLVMContextRef C,
                                                      LLVMCrabLangAttribute CrabLangAttr) {
  return wrap(Attribute::get(*unwrap(C), fromCrabLang(CrabLangAttr)));
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateAlignmentAttr(LLVMContextRef C,
                                                        uint64_t Bytes) {
  return wrap(Attribute::getWithAlignment(*unwrap(C), llvm::Align(Bytes)));
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateDereferenceableAttr(LLVMContextRef C,
                                                              uint64_t Bytes) {
  return wrap(Attribute::getWithDereferenceableBytes(*unwrap(C), Bytes));
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateDereferenceableOrNullAttr(LLVMContextRef C,
                                                                    uint64_t Bytes) {
  return wrap(Attribute::getWithDereferenceableOrNullBytes(*unwrap(C), Bytes));
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateByValAttr(LLVMContextRef C, LLVMTypeRef Ty) {
  return wrap(Attribute::getWithByValType(*unwrap(C), unwrap(Ty)));
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateStructRetAttr(LLVMContextRef C, LLVMTypeRef Ty) {
  return wrap(Attribute::getWithStructRetType(*unwrap(C), unwrap(Ty)));
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateElementTypeAttr(LLVMContextRef C, LLVMTypeRef Ty) {
#if LLVM_VERSION_GE(15, 0)
  return wrap(Attribute::get(*unwrap(C), Attribute::ElementType, unwrap(Ty)));
#else
  report_fatal_error("Should not be needed on LLVM < 15");
#endif
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateUWTableAttr(LLVMContextRef C, bool Async) {
#if LLVM_VERSION_LT(15, 0)
  return wrap(Attribute::get(*unwrap(C), Attribute::UWTable));
#else
  return wrap(Attribute::getWithUWTableKind(
      *unwrap(C), Async ? UWTableKind::Async : UWTableKind::Sync));
#endif
}

extern "C" LLVMAttributeRef LLVMCrabLangCreateAllocSizeAttr(LLVMContextRef C, uint32_t ElementSizeArg) {
  return wrap(Attribute::getWithAllocSizeArgs(*unwrap(C), ElementSizeArg,
#if LLVM_VERSION_LT(16, 0)
                                              None
#else
                                              std::nullopt
#endif
                                              ));
}

#if LLVM_VERSION_GE(15, 0)

// These values **must** match ffi::AllocKindFlags.
// It _happens_ to match the LLVM values of llvm::AllocFnKind,
// but that's happenstance and we do explicit conversions before
// passing them to LLVM.
enum class LLVMCrabLangAllocKindFlags : uint64_t {
  Unknown = 0,
  Alloc = 1,
  Realloc = 1 << 1,
  Free = 1 << 2,
  Uninitialized = 1 << 3,
  Zeroed = 1 << 4,
  Aligned = 1 << 5,
};

static LLVMCrabLangAllocKindFlags operator&(LLVMCrabLangAllocKindFlags A, LLVMCrabLangAllocKindFlags B) {
  return static_cast<LLVMCrabLangAllocKindFlags>(static_cast<uint64_t>(A) &
                                      static_cast<uint64_t>(B));
}

static bool isSet(LLVMCrabLangAllocKindFlags F) { return F != LLVMCrabLangAllocKindFlags::Unknown; }

static llvm::AllocFnKind allocKindFromCrabLang(LLVMCrabLangAllocKindFlags F) {
  llvm::AllocFnKind AFK = llvm::AllocFnKind::Unknown;
  if (isSet(F & LLVMCrabLangAllocKindFlags::Alloc)) {
    AFK |= llvm::AllocFnKind::Alloc;
  }
  if (isSet(F & LLVMCrabLangAllocKindFlags::Realloc)) {
    AFK |= llvm::AllocFnKind::Realloc;
  }
  if (isSet(F & LLVMCrabLangAllocKindFlags::Free)) {
    AFK |= llvm::AllocFnKind::Free;
  }
  if (isSet(F & LLVMCrabLangAllocKindFlags::Uninitialized)) {
    AFK |= llvm::AllocFnKind::Uninitialized;
  }
  if (isSet(F & LLVMCrabLangAllocKindFlags::Zeroed)) {
    AFK |= llvm::AllocFnKind::Zeroed;
  }
  if (isSet(F & LLVMCrabLangAllocKindFlags::Aligned)) {
    AFK |= llvm::AllocFnKind::Aligned;
  }
  return AFK;
}
#endif

extern "C" LLVMAttributeRef LLVMCrabLangCreateAllocKindAttr(LLVMContextRef C, uint64_t AllocKindArg) {
#if LLVM_VERSION_GE(15, 0)
  return wrap(Attribute::get(*unwrap(C), Attribute::AllocKind,
      static_cast<uint64_t>(allocKindFromCrabLang(static_cast<LLVMCrabLangAllocKindFlags>(AllocKindArg)))));
#else
  report_fatal_error(
      "allockind attributes are new in LLVM 15 and should not be used on older LLVMs");
#endif
}

// Simplified representation of `MemoryEffects` across the FFI boundary.
//
// Each variant corresponds to one of the static factory methods on `MemoryEffects`.
enum class LLVMCrabLangMemoryEffects {
  None,
  ReadOnly,
  InaccessibleMemOnly,
};

extern "C" LLVMAttributeRef LLVMCrabLangCreateMemoryEffectsAttr(LLVMContextRef C,
                                                            LLVMCrabLangMemoryEffects Effects) {
#if LLVM_VERSION_GE(16, 0)
  switch (Effects) {
    case LLVMCrabLangMemoryEffects::None:
      return wrap(Attribute::getWithMemoryEffects(*unwrap(C), MemoryEffects::none()));
    case LLVMCrabLangMemoryEffects::ReadOnly:
      return wrap(Attribute::getWithMemoryEffects(*unwrap(C), MemoryEffects::readOnly()));
    case LLVMCrabLangMemoryEffects::InaccessibleMemOnly:
      return wrap(Attribute::getWithMemoryEffects(*unwrap(C),
                                                  MemoryEffects::inaccessibleMemOnly()));
    default:
      report_fatal_error("bad MemoryEffects.");
  }
#else
  switch (Effects) {
    case LLVMCrabLangMemoryEffects::None:
      return wrap(Attribute::get(*unwrap(C), Attribute::ReadNone));
    case LLVMCrabLangMemoryEffects::ReadOnly:
      return wrap(Attribute::get(*unwrap(C), Attribute::ReadOnly));
    case LLVMCrabLangMemoryEffects::InaccessibleMemOnly:
      return wrap(Attribute::get(*unwrap(C), Attribute::InaccessibleMemOnly));
    default:
      report_fatal_error("bad MemoryEffects.");
  }
#endif
}

// Enable a fast-math flag
//
// https://llvm.org/docs/LangRef.html#fast-math-flags
extern "C" void LLVMCrabLangSetFastMath(LLVMValueRef V) {
  if (auto I = dyn_cast<Instruction>(unwrap<Value>(V))) {
    I->setFast(true);
  }
}

extern "C" LLVMValueRef
LLVMCrabLangBuildAtomicLoad(LLVMBuilderRef B, LLVMTypeRef Ty, LLVMValueRef Source,
                        const char *Name, LLVMAtomicOrdering Order) {
  Value *Ptr = unwrap(Source);
  LoadInst *LI = unwrap(B)->CreateLoad(unwrap(Ty), Ptr, Name);
  LI->setAtomic(fromCrabLang(Order));
  return wrap(LI);
}

extern "C" LLVMValueRef LLVMCrabLangBuildAtomicStore(LLVMBuilderRef B,
                                                 LLVMValueRef V,
                                                 LLVMValueRef Target,
                                                 LLVMAtomicOrdering Order) {
  StoreInst *SI = unwrap(B)->CreateStore(unwrap(V), unwrap(Target));
  SI->setAtomic(fromCrabLang(Order));
  return wrap(SI);
}

enum class LLVMCrabLangAsmDialect {
  Att,
  Intel,
};

static InlineAsm::AsmDialect fromCrabLang(LLVMCrabLangAsmDialect Dialect) {
  switch (Dialect) {
  case LLVMCrabLangAsmDialect::Att:
    return InlineAsm::AD_ATT;
  case LLVMCrabLangAsmDialect::Intel:
    return InlineAsm::AD_Intel;
  default:
    report_fatal_error("bad AsmDialect.");
  }
}

extern "C" LLVMValueRef
LLVMCrabLangInlineAsm(LLVMTypeRef Ty, char *AsmString, size_t AsmStringLen,
                  char *Constraints, size_t ConstraintsLen,
                  LLVMBool HasSideEffects, LLVMBool IsAlignStack,
                  LLVMCrabLangAsmDialect Dialect, LLVMBool CanThrow) {
  return wrap(InlineAsm::get(unwrap<FunctionType>(Ty),
                             StringRef(AsmString, AsmStringLen),
                             StringRef(Constraints, ConstraintsLen),
                             HasSideEffects, IsAlignStack,
                             fromCrabLang(Dialect), CanThrow));
}

extern "C" bool LLVMCrabLangInlineAsmVerify(LLVMTypeRef Ty, char *Constraints,
                                        size_t ConstraintsLen) {
#if LLVM_VERSION_LT(15, 0)
  return InlineAsm::Verify(unwrap<FunctionType>(Ty),
                           StringRef(Constraints, ConstraintsLen));
#else
  // llvm::Error converts to true if it is an error.
  return !llvm::errorToBool(InlineAsm::verify(
      unwrap<FunctionType>(Ty), StringRef(Constraints, ConstraintsLen)));
#endif
}

typedef DIBuilder *LLVMCrabLangDIBuilderRef;

template <typename DIT> DIT *unwrapDIPtr(LLVMMetadataRef Ref) {
  return (DIT *)(Ref ? unwrap<MDNode>(Ref) : nullptr);
}

#define DIDescriptor DIScope
#define DIArray DINodeArray
#define unwrapDI unwrapDIPtr

// These values **must** match debuginfo::DIFlags! They also *happen*
// to match LLVM, but that isn't required as we do giant sets of
// matching below. The value shouldn't be directly passed to LLVM.
enum class LLVMCrabLangDIFlags : uint32_t {
  FlagZero = 0,
  FlagPrivate = 1,
  FlagProtected = 2,
  FlagPublic = 3,
  FlagFwdDecl = (1 << 2),
  FlagAppleBlock = (1 << 3),
  FlagBlockByrefStruct = (1 << 4),
  FlagVirtual = (1 << 5),
  FlagArtificial = (1 << 6),
  FlagExplicit = (1 << 7),
  FlagPrototyped = (1 << 8),
  FlagObjcClassComplete = (1 << 9),
  FlagObjectPointer = (1 << 10),
  FlagVector = (1 << 11),
  FlagStaticMember = (1 << 12),
  FlagLValueReference = (1 << 13),
  FlagRValueReference = (1 << 14),
  FlagExternalTypeRef = (1 << 15),
  FlagIntroducedVirtual = (1 << 18),
  FlagBitField = (1 << 19),
  FlagNoReturn = (1 << 20),
  // Do not add values that are not supported by the minimum LLVM
  // version we support! see llvm/include/llvm/IR/DebugInfoFlags.def
};

inline LLVMCrabLangDIFlags operator&(LLVMCrabLangDIFlags A, LLVMCrabLangDIFlags B) {
  return static_cast<LLVMCrabLangDIFlags>(static_cast<uint32_t>(A) &
                                      static_cast<uint32_t>(B));
}

inline LLVMCrabLangDIFlags operator|(LLVMCrabLangDIFlags A, LLVMCrabLangDIFlags B) {
  return static_cast<LLVMCrabLangDIFlags>(static_cast<uint32_t>(A) |
                                      static_cast<uint32_t>(B));
}

inline LLVMCrabLangDIFlags &operator|=(LLVMCrabLangDIFlags &A, LLVMCrabLangDIFlags B) {
  return A = A | B;
}

inline bool isSet(LLVMCrabLangDIFlags F) { return F != LLVMCrabLangDIFlags::FlagZero; }

inline LLVMCrabLangDIFlags visibility(LLVMCrabLangDIFlags F) {
  return static_cast<LLVMCrabLangDIFlags>(static_cast<uint32_t>(F) & 0x3);
}

static DINode::DIFlags fromCrabLang(LLVMCrabLangDIFlags Flags) {
  DINode::DIFlags Result = DINode::DIFlags::FlagZero;

  switch (visibility(Flags)) {
  case LLVMCrabLangDIFlags::FlagPrivate:
    Result |= DINode::DIFlags::FlagPrivate;
    break;
  case LLVMCrabLangDIFlags::FlagProtected:
    Result |= DINode::DIFlags::FlagProtected;
    break;
  case LLVMCrabLangDIFlags::FlagPublic:
    Result |= DINode::DIFlags::FlagPublic;
    break;
  default:
    // The rest are handled below
    break;
  }

  if (isSet(Flags & LLVMCrabLangDIFlags::FlagFwdDecl)) {
    Result |= DINode::DIFlags::FlagFwdDecl;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagAppleBlock)) {
    Result |= DINode::DIFlags::FlagAppleBlock;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagVirtual)) {
    Result |= DINode::DIFlags::FlagVirtual;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagArtificial)) {
    Result |= DINode::DIFlags::FlagArtificial;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagExplicit)) {
    Result |= DINode::DIFlags::FlagExplicit;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagPrototyped)) {
    Result |= DINode::DIFlags::FlagPrototyped;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagObjcClassComplete)) {
    Result |= DINode::DIFlags::FlagObjcClassComplete;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagObjectPointer)) {
    Result |= DINode::DIFlags::FlagObjectPointer;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagVector)) {
    Result |= DINode::DIFlags::FlagVector;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagStaticMember)) {
    Result |= DINode::DIFlags::FlagStaticMember;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagLValueReference)) {
    Result |= DINode::DIFlags::FlagLValueReference;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagRValueReference)) {
    Result |= DINode::DIFlags::FlagRValueReference;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagIntroducedVirtual)) {
    Result |= DINode::DIFlags::FlagIntroducedVirtual;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagBitField)) {
    Result |= DINode::DIFlags::FlagBitField;
  }
  if (isSet(Flags & LLVMCrabLangDIFlags::FlagNoReturn)) {
    Result |= DINode::DIFlags::FlagNoReturn;
  }

  return Result;
}

// These values **must** match debuginfo::DISPFlags! They also *happen*
// to match LLVM, but that isn't required as we do giant sets of
// matching below. The value shouldn't be directly passed to LLVM.
enum class LLVMCrabLangDISPFlags : uint32_t {
  SPFlagZero = 0,
  SPFlagVirtual = 1,
  SPFlagPureVirtual = 2,
  SPFlagLocalToUnit = (1 << 2),
  SPFlagDefinition = (1 << 3),
  SPFlagOptimized = (1 << 4),
  SPFlagMainSubprogram = (1 << 5),
  // Do not add values that are not supported by the minimum LLVM
  // version we support! see llvm/include/llvm/IR/DebugInfoFlags.def
  // (In LLVM < 8, createFunction supported these as separate bool arguments.)
};

inline LLVMCrabLangDISPFlags operator&(LLVMCrabLangDISPFlags A, LLVMCrabLangDISPFlags B) {
  return static_cast<LLVMCrabLangDISPFlags>(static_cast<uint32_t>(A) &
                                      static_cast<uint32_t>(B));
}

inline LLVMCrabLangDISPFlags operator|(LLVMCrabLangDISPFlags A, LLVMCrabLangDISPFlags B) {
  return static_cast<LLVMCrabLangDISPFlags>(static_cast<uint32_t>(A) |
                                      static_cast<uint32_t>(B));
}

inline LLVMCrabLangDISPFlags &operator|=(LLVMCrabLangDISPFlags &A, LLVMCrabLangDISPFlags B) {
  return A = A | B;
}

inline bool isSet(LLVMCrabLangDISPFlags F) { return F != LLVMCrabLangDISPFlags::SPFlagZero; }

inline LLVMCrabLangDISPFlags virtuality(LLVMCrabLangDISPFlags F) {
  return static_cast<LLVMCrabLangDISPFlags>(static_cast<uint32_t>(F) & 0x3);
}

static DISubprogram::DISPFlags fromCrabLang(LLVMCrabLangDISPFlags SPFlags) {
  DISubprogram::DISPFlags Result = DISubprogram::DISPFlags::SPFlagZero;

  switch (virtuality(SPFlags)) {
  case LLVMCrabLangDISPFlags::SPFlagVirtual:
    Result |= DISubprogram::DISPFlags::SPFlagVirtual;
    break;
  case LLVMCrabLangDISPFlags::SPFlagPureVirtual:
    Result |= DISubprogram::DISPFlags::SPFlagPureVirtual;
    break;
  default:
    // The rest are handled below
    break;
  }

  if (isSet(SPFlags & LLVMCrabLangDISPFlags::SPFlagLocalToUnit)) {
    Result |= DISubprogram::DISPFlags::SPFlagLocalToUnit;
  }
  if (isSet(SPFlags & LLVMCrabLangDISPFlags::SPFlagDefinition)) {
    Result |= DISubprogram::DISPFlags::SPFlagDefinition;
  }
  if (isSet(SPFlags & LLVMCrabLangDISPFlags::SPFlagOptimized)) {
    Result |= DISubprogram::DISPFlags::SPFlagOptimized;
  }
  if (isSet(SPFlags & LLVMCrabLangDISPFlags::SPFlagMainSubprogram)) {
    Result |= DISubprogram::DISPFlags::SPFlagMainSubprogram;
  }

  return Result;
}

enum class LLVMCrabLangDebugEmissionKind {
  NoDebug,
  FullDebug,
  LineTablesOnly,
  DebugDirectivesOnly,
};

static DICompileUnit::DebugEmissionKind fromCrabLang(LLVMCrabLangDebugEmissionKind Kind) {
  switch (Kind) {
  case LLVMCrabLangDebugEmissionKind::NoDebug:
    return DICompileUnit::DebugEmissionKind::NoDebug;
  case LLVMCrabLangDebugEmissionKind::FullDebug:
    return DICompileUnit::DebugEmissionKind::FullDebug;
  case LLVMCrabLangDebugEmissionKind::LineTablesOnly:
    return DICompileUnit::DebugEmissionKind::LineTablesOnly;
  case LLVMCrabLangDebugEmissionKind::DebugDirectivesOnly:
    return DICompileUnit::DebugEmissionKind::DebugDirectivesOnly;
  default:
    report_fatal_error("bad DebugEmissionKind.");
  }
}

enum class LLVMCrabLangChecksumKind {
  None,
  MD5,
  SHA1,
  SHA256,
};

#if LLVM_VERSION_LT(16, 0)
static Optional<DIFile::ChecksumKind> fromCrabLang(LLVMCrabLangChecksumKind Kind) {
#else
static std::optional<DIFile::ChecksumKind> fromCrabLang(LLVMCrabLangChecksumKind Kind) {
#endif
  switch (Kind) {
  case LLVMCrabLangChecksumKind::None:
#if LLVM_VERSION_LT(16, 0)
    return None;
#else
    return std::nullopt;
#endif
  case LLVMCrabLangChecksumKind::MD5:
    return DIFile::ChecksumKind::CSK_MD5;
  case LLVMCrabLangChecksumKind::SHA1:
    return DIFile::ChecksumKind::CSK_SHA1;
  case LLVMCrabLangChecksumKind::SHA256:
    return DIFile::ChecksumKind::CSK_SHA256;
  default:
    report_fatal_error("bad ChecksumKind.");
  }
}

extern "C" uint32_t LLVMCrabLangDebugMetadataVersion() {
  return DEBUG_METADATA_VERSION;
}

extern "C" uint32_t LLVMCrabLangVersionPatch() { return LLVM_VERSION_PATCH; }

extern "C" uint32_t LLVMCrabLangVersionMinor() { return LLVM_VERSION_MINOR; }

extern "C" uint32_t LLVMCrabLangVersionMajor() { return LLVM_VERSION_MAJOR; }

extern "C" void LLVMCrabLangAddModuleFlag(
    LLVMModuleRef M,
    Module::ModFlagBehavior MergeBehavior,
    const char *Name,
    uint32_t Value) {
  unwrap(M)->addModuleFlag(MergeBehavior, Name, Value);
}

extern "C" bool LLVMCrabLangHasModuleFlag(LLVMModuleRef M, const char *Name,
                                      size_t Len) {
  return unwrap(M)->getModuleFlag(StringRef(Name, Len)) != nullptr;
}

extern "C" void LLVMCrabLangGlobalAddMetadata(
    LLVMValueRef Global, unsigned Kind, LLVMMetadataRef MD) {
  unwrap<GlobalObject>(Global)->addMetadata(Kind, *unwrap<MDNode>(MD));
}

extern "C" LLVMCrabLangDIBuilderRef LLVMCrabLangDIBuilderCreate(LLVMModuleRef M) {
  return new DIBuilder(*unwrap(M));
}

extern "C" void LLVMCrabLangDIBuilderDispose(LLVMCrabLangDIBuilderRef Builder) {
  delete Builder;
}

extern "C" void LLVMCrabLangDIBuilderFinalize(LLVMCrabLangDIBuilderRef Builder) {
  Builder->finalize();
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateCompileUnit(
    LLVMCrabLangDIBuilderRef Builder, unsigned Lang, LLVMMetadataRef FileRef,
    const char *Producer, size_t ProducerLen, bool isOptimized,
    const char *Flags, unsigned RuntimeVer,
    const char *SplitName, size_t SplitNameLen,
    LLVMCrabLangDebugEmissionKind Kind,
    uint64_t DWOId, bool SplitDebugInlining) {
  auto *File = unwrapDI<DIFile>(FileRef);

  return wrap(Builder->createCompileUnit(Lang, File, StringRef(Producer, ProducerLen),
                                         isOptimized, Flags, RuntimeVer,
                                         StringRef(SplitName, SplitNameLen),
                                         fromCrabLang(Kind), DWOId, SplitDebugInlining));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateFile(
    LLVMCrabLangDIBuilderRef Builder,
    const char *Filename, size_t FilenameLen,
    const char *Directory, size_t DirectoryLen, LLVMCrabLangChecksumKind CSKind,
    const char *Checksum, size_t ChecksumLen) {

#if LLVM_VERSION_LT(16, 0)
  Optional<DIFile::ChecksumKind> llvmCSKind = fromCrabLang(CSKind);
#else
  std::optional<DIFile::ChecksumKind> llvmCSKind = fromCrabLang(CSKind);
#endif

#if LLVM_VERSION_LT(16, 0)
  Optional<DIFile::ChecksumInfo<StringRef>> CSInfo{};
#else
  std::optional<DIFile::ChecksumInfo<StringRef>> CSInfo{};
#endif
  if (llvmCSKind)
    CSInfo.emplace(*llvmCSKind, StringRef{Checksum, ChecksumLen});
  return wrap(Builder->createFile(StringRef(Filename, FilenameLen),
                                  StringRef(Directory, DirectoryLen),
                                  CSInfo));
}

extern "C" LLVMMetadataRef
LLVMCrabLangDIBuilderCreateSubroutineType(LLVMCrabLangDIBuilderRef Builder,
                                      LLVMMetadataRef ParameterTypes) {
  return wrap(Builder->createSubroutineType(
      DITypeRefArray(unwrap<MDTuple>(ParameterTypes))));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateFunction(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    const char *LinkageName, size_t LinkageNameLen,
    LLVMMetadataRef File, unsigned LineNo,
    LLVMMetadataRef Ty, unsigned ScopeLine, LLVMCrabLangDIFlags Flags,
    LLVMCrabLangDISPFlags SPFlags, LLVMValueRef MaybeFn, LLVMMetadataRef TParam,
    LLVMMetadataRef Decl) {
  DITemplateParameterArray TParams =
      DITemplateParameterArray(unwrap<MDTuple>(TParam));
  DISubprogram::DISPFlags llvmSPFlags = fromCrabLang(SPFlags);
  DINode::DIFlags llvmFlags = fromCrabLang(Flags);
  DISubprogram *Sub = Builder->createFunction(
      unwrapDI<DIScope>(Scope),
      StringRef(Name, NameLen),
      StringRef(LinkageName, LinkageNameLen),
      unwrapDI<DIFile>(File), LineNo,
      unwrapDI<DISubroutineType>(Ty), ScopeLine, llvmFlags,
      llvmSPFlags, TParams, unwrapDIPtr<DISubprogram>(Decl));
  if (MaybeFn)
    unwrap<Function>(MaybeFn)->setSubprogram(Sub);
  return wrap(Sub);
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateBasicType(
    LLVMCrabLangDIBuilderRef Builder, const char *Name, size_t NameLen,
    uint64_t SizeInBits, unsigned Encoding) {
  return wrap(Builder->createBasicType(StringRef(Name, NameLen), SizeInBits, Encoding));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateTypedef(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Type, const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNo, LLVMMetadataRef Scope) {
  return wrap(Builder->createTypedef(
    unwrap<DIType>(Type), StringRef(Name, NameLen), unwrap<DIFile>(File),
    LineNo, unwrapDIPtr<DIScope>(Scope)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreatePointerType(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef PointeeTy,
    uint64_t SizeInBits, uint32_t AlignInBits, unsigned AddressSpace,
    const char *Name, size_t NameLen) {
  return wrap(Builder->createPointerType(unwrapDI<DIType>(PointeeTy),
                                         SizeInBits, AlignInBits,
                                         AddressSpace,
                                         StringRef(Name, NameLen)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateStructType(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNumber, uint64_t SizeInBits,
    uint32_t AlignInBits, LLVMCrabLangDIFlags Flags,
    LLVMMetadataRef DerivedFrom, LLVMMetadataRef Elements,
    unsigned RunTimeLang, LLVMMetadataRef VTableHolder,
    const char *UniqueId, size_t UniqueIdLen) {
  return wrap(Builder->createStructType(
      unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen),
      unwrapDI<DIFile>(File), LineNumber,
      SizeInBits, AlignInBits, fromCrabLang(Flags), unwrapDI<DIType>(DerivedFrom),
      DINodeArray(unwrapDI<MDTuple>(Elements)), RunTimeLang,
      unwrapDI<DIType>(VTableHolder), StringRef(UniqueId, UniqueIdLen)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateVariantPart(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNumber, uint64_t SizeInBits,
    uint32_t AlignInBits, LLVMCrabLangDIFlags Flags, LLVMMetadataRef Discriminator,
    LLVMMetadataRef Elements, const char *UniqueId, size_t UniqueIdLen) {
  return wrap(Builder->createVariantPart(
      unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen),
      unwrapDI<DIFile>(File), LineNumber,
      SizeInBits, AlignInBits, fromCrabLang(Flags), unwrapDI<DIDerivedType>(Discriminator),
      DINodeArray(unwrapDI<MDTuple>(Elements)), StringRef(UniqueId, UniqueIdLen)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateMemberType(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNo, uint64_t SizeInBits,
    uint32_t AlignInBits, uint64_t OffsetInBits, LLVMCrabLangDIFlags Flags,
    LLVMMetadataRef Ty) {
  return wrap(Builder->createMemberType(unwrapDI<DIDescriptor>(Scope),
                                        StringRef(Name, NameLen),
                                        unwrapDI<DIFile>(File), LineNo,
                                        SizeInBits, AlignInBits, OffsetInBits,
                                        fromCrabLang(Flags), unwrapDI<DIType>(Ty)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateVariantMemberType(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen, LLVMMetadataRef File, unsigned LineNo,
    uint64_t SizeInBits, uint32_t AlignInBits, uint64_t OffsetInBits, LLVMValueRef Discriminant,
    LLVMCrabLangDIFlags Flags, LLVMMetadataRef Ty) {
  llvm::ConstantInt* D = nullptr;
  if (Discriminant) {
    D = unwrap<llvm::ConstantInt>(Discriminant);
  }
  return wrap(Builder->createVariantMemberType(unwrapDI<DIDescriptor>(Scope),
                                               StringRef(Name, NameLen),
                                               unwrapDI<DIFile>(File), LineNo,
                                               SizeInBits, AlignInBits, OffsetInBits, D,
                                               fromCrabLang(Flags), unwrapDI<DIType>(Ty)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateStaticMemberType(
    LLVMCrabLangDIBuilderRef Builder,
    LLVMMetadataRef Scope,
    const char *Name,
    size_t NameLen,
    LLVMMetadataRef File,
    unsigned LineNo,
    LLVMMetadataRef Ty,
    LLVMCrabLangDIFlags Flags,
    LLVMValueRef val,
    uint32_t AlignInBits
) {
  return wrap(Builder->createStaticMemberType(
    unwrapDI<DIDescriptor>(Scope),
    StringRef(Name, NameLen),
    unwrapDI<DIFile>(File),
    LineNo,
    unwrapDI<DIType>(Ty),
    fromCrabLang(Flags),
    unwrap<llvm::ConstantInt>(val),
    AlignInBits
  ));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateLexicalBlock(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    LLVMMetadataRef File, unsigned Line, unsigned Col) {
  return wrap(Builder->createLexicalBlock(unwrapDI<DIDescriptor>(Scope),
                                          unwrapDI<DIFile>(File), Line, Col));
}

extern "C" LLVMMetadataRef
LLVMCrabLangDIBuilderCreateLexicalBlockFile(LLVMCrabLangDIBuilderRef Builder,
                                        LLVMMetadataRef Scope,
                                        LLVMMetadataRef File) {
  return wrap(Builder->createLexicalBlockFile(unwrapDI<DIDescriptor>(Scope),
                                              unwrapDI<DIFile>(File)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateStaticVariable(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Context,
    const char *Name, size_t NameLen,
    const char *LinkageName, size_t LinkageNameLen,
    LLVMMetadataRef File, unsigned LineNo,
    LLVMMetadataRef Ty, bool IsLocalToUnit, LLVMValueRef V,
    LLVMMetadataRef Decl = nullptr, uint32_t AlignInBits = 0) {
  llvm::GlobalVariable *InitVal = cast<llvm::GlobalVariable>(unwrap(V));

  llvm::DIExpression *InitExpr = nullptr;
  if (llvm::ConstantInt *IntVal = llvm::dyn_cast<llvm::ConstantInt>(InitVal)) {
    InitExpr = Builder->createConstantValueExpression(
        IntVal->getValue().getSExtValue());
  } else if (llvm::ConstantFP *FPVal =
                 llvm::dyn_cast<llvm::ConstantFP>(InitVal)) {
    InitExpr = Builder->createConstantValueExpression(
        FPVal->getValueAPF().bitcastToAPInt().getZExtValue());
  }

  llvm::DIGlobalVariableExpression *VarExpr = Builder->createGlobalVariableExpression(
      unwrapDI<DIDescriptor>(Context), StringRef(Name, NameLen),
      StringRef(LinkageName, LinkageNameLen),
      unwrapDI<DIFile>(File), LineNo, unwrapDI<DIType>(Ty), IsLocalToUnit,
      /* isDefined */ true,
      InitExpr, unwrapDIPtr<MDNode>(Decl),
      /* templateParams */ nullptr,
      AlignInBits);

  InitVal->setMetadata("dbg", VarExpr);

  return wrap(VarExpr);
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateVariable(
    LLVMCrabLangDIBuilderRef Builder, unsigned Tag, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNo,
    LLVMMetadataRef Ty, bool AlwaysPreserve, LLVMCrabLangDIFlags Flags,
    unsigned ArgNo, uint32_t AlignInBits) {
  if (Tag == 0x100) { // DW_TAG_auto_variable
    return wrap(Builder->createAutoVariable(
        unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen),
        unwrapDI<DIFile>(File), LineNo,
        unwrapDI<DIType>(Ty), AlwaysPreserve, fromCrabLang(Flags), AlignInBits));
  } else {
    return wrap(Builder->createParameterVariable(
        unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen), ArgNo,
        unwrapDI<DIFile>(File), LineNo,
        unwrapDI<DIType>(Ty), AlwaysPreserve, fromCrabLang(Flags)));
  }
}

extern "C" LLVMMetadataRef
LLVMCrabLangDIBuilderCreateArrayType(LLVMCrabLangDIBuilderRef Builder, uint64_t Size,
                                 uint32_t AlignInBits, LLVMMetadataRef Ty,
                                 LLVMMetadataRef Subscripts) {
  return wrap(
      Builder->createArrayType(Size, AlignInBits, unwrapDI<DIType>(Ty),
                               DINodeArray(unwrapDI<MDTuple>(Subscripts))));
}

extern "C" LLVMMetadataRef
LLVMCrabLangDIBuilderGetOrCreateSubrange(LLVMCrabLangDIBuilderRef Builder, int64_t Lo,
                                     int64_t Count) {
  return wrap(Builder->getOrCreateSubrange(Lo, Count));
}

extern "C" LLVMMetadataRef
LLVMCrabLangDIBuilderGetOrCreateArray(LLVMCrabLangDIBuilderRef Builder,
                                  LLVMMetadataRef *Ptr, unsigned Count) {
  Metadata **DataValue = unwrap(Ptr);
  return wrap(
      Builder->getOrCreateArray(ArrayRef<Metadata *>(DataValue, Count)).get());
}

extern "C" LLVMValueRef LLVMCrabLangDIBuilderInsertDeclareAtEnd(
    LLVMCrabLangDIBuilderRef Builder, LLVMValueRef V, LLVMMetadataRef VarInfo,
    uint64_t *AddrOps, unsigned AddrOpsCount, LLVMMetadataRef DL,
    LLVMBasicBlockRef InsertAtEnd) {
  return wrap(Builder->insertDeclare(
      unwrap(V), unwrap<DILocalVariable>(VarInfo),
      Builder->createExpression(llvm::ArrayRef<uint64_t>(AddrOps, AddrOpsCount)),
      DebugLoc(cast<MDNode>(unwrap(DL))),
      unwrap(InsertAtEnd)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateEnumerator(
    LLVMCrabLangDIBuilderRef Builder, const char *Name, size_t NameLen,
    const uint64_t Value[2], unsigned SizeInBits, bool IsUnsigned) {
  return wrap(Builder->createEnumerator(StringRef(Name, NameLen),
      APSInt(APInt(SizeInBits, ArrayRef<uint64_t>(Value, 2)), IsUnsigned)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateEnumerationType(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNumber, uint64_t SizeInBits,
    uint32_t AlignInBits, LLVMMetadataRef Elements,
    LLVMMetadataRef ClassTy, bool IsScoped) {
  return wrap(Builder->createEnumerationType(
      unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen),
      unwrapDI<DIFile>(File), LineNumber,
      SizeInBits, AlignInBits, DINodeArray(unwrapDI<MDTuple>(Elements)),
      unwrapDI<DIType>(ClassTy), "", IsScoped));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateUnionType(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen,
    LLVMMetadataRef File, unsigned LineNumber, uint64_t SizeInBits,
    uint32_t AlignInBits, LLVMCrabLangDIFlags Flags, LLVMMetadataRef Elements,
    unsigned RunTimeLang, const char *UniqueId, size_t UniqueIdLen) {
  return wrap(Builder->createUnionType(
      unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen), unwrapDI<DIFile>(File),
      LineNumber, SizeInBits, AlignInBits, fromCrabLang(Flags),
      DINodeArray(unwrapDI<MDTuple>(Elements)), RunTimeLang,
      StringRef(UniqueId, UniqueIdLen)));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateTemplateTypeParameter(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen, LLVMMetadataRef Ty) {
  bool IsDefault = false; // FIXME: should we ever set this true?
  return wrap(Builder->createTemplateTypeParameter(
      unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen), unwrapDI<DIType>(Ty), IsDefault));
}

extern "C" LLVMMetadataRef LLVMCrabLangDIBuilderCreateNameSpace(
    LLVMCrabLangDIBuilderRef Builder, LLVMMetadataRef Scope,
    const char *Name, size_t NameLen, bool ExportSymbols) {
  return wrap(Builder->createNameSpace(
      unwrapDI<DIDescriptor>(Scope), StringRef(Name, NameLen), ExportSymbols
  ));
}

extern "C" void
LLVMCrabLangDICompositeTypeReplaceArrays(LLVMCrabLangDIBuilderRef Builder,
                                     LLVMMetadataRef CompositeTy,
                                     LLVMMetadataRef Elements,
                                     LLVMMetadataRef Params) {
  DICompositeType *Tmp = unwrapDI<DICompositeType>(CompositeTy);
  Builder->replaceArrays(Tmp, DINodeArray(unwrap<MDTuple>(Elements)),
                         DINodeArray(unwrap<MDTuple>(Params)));
}

extern "C" LLVMMetadataRef
LLVMCrabLangDIBuilderCreateDebugLocation(unsigned Line, unsigned Column,
                                     LLVMMetadataRef ScopeRef,
                                     LLVMMetadataRef InlinedAt) {
  MDNode *Scope = unwrapDIPtr<MDNode>(ScopeRef);
  DILocation *Loc = DILocation::get(
      Scope->getContext(), Line, Column, Scope,
      unwrapDIPtr<MDNode>(InlinedAt));
  return wrap(Loc);
}

extern "C" uint64_t LLVMCrabLangDIBuilderCreateOpDeref() {
  return dwarf::DW_OP_deref;
}

extern "C" uint64_t LLVMCrabLangDIBuilderCreateOpPlusUconst() {
  return dwarf::DW_OP_plus_uconst;
}

extern "C" int64_t LLVMCrabLangDIBuilderCreateOpLLVMFragment() {
  return dwarf::DW_OP_LLVM_fragment;
}

extern "C" void LLVMCrabLangWriteTypeToString(LLVMTypeRef Ty, CrabLangStringRef Str) {
  RawCrabLangStringOstream OS(Str);
  unwrap<llvm::Type>(Ty)->print(OS);
}

extern "C" void LLVMCrabLangWriteValueToString(LLVMValueRef V,
                                           CrabLangStringRef Str) {
  RawCrabLangStringOstream OS(Str);
  if (!V) {
    OS << "(null)";
  } else {
    OS << "(";
    unwrap<llvm::Value>(V)->getType()->print(OS);
    OS << ":";
    unwrap<llvm::Value>(V)->print(OS);
    OS << ")";
  }
}

// LLVMArrayType function does not support 64-bit ElementCount
// FIXME: replace with LLVMArrayType2 when bumped minimal version to llvm-17
// https://github.com/llvm/llvm-project/commit/35276f16e5a2cae0dfb49c0fbf874d4d2f177acc
extern "C" LLVMTypeRef LLVMCrabLangArrayType(LLVMTypeRef ElementTy,
                                         uint64_t ElementCount) {
  return wrap(ArrayType::get(unwrap(ElementTy), ElementCount));
}

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(Twine, LLVMTwineRef)

extern "C" void LLVMCrabLangWriteTwineToString(LLVMTwineRef T, CrabLangStringRef Str) {
  RawCrabLangStringOstream OS(Str);
  unwrap(T)->print(OS);
}

extern "C" void LLVMCrabLangUnpackOptimizationDiagnostic(
    LLVMDiagnosticInfoRef DI, CrabLangStringRef PassNameOut,
    LLVMValueRef *FunctionOut, unsigned* Line, unsigned* Column,
    CrabLangStringRef FilenameOut, CrabLangStringRef MessageOut) {
  // Undefined to call this not on an optimization diagnostic!
  llvm::DiagnosticInfoOptimizationBase *Opt =
      static_cast<llvm::DiagnosticInfoOptimizationBase *>(unwrap(DI));

  RawCrabLangStringOstream PassNameOS(PassNameOut);
  PassNameOS << Opt->getPassName();
  *FunctionOut = wrap(&Opt->getFunction());

  RawCrabLangStringOstream FilenameOS(FilenameOut);
  DiagnosticLocation loc = Opt->getLocation();
  if (loc.isValid()) {
    *Line = loc.getLine();
    *Column = loc.getColumn();
    FilenameOS << loc.getAbsolutePath();
  }

  RawCrabLangStringOstream MessageOS(MessageOut);
  MessageOS << Opt->getMsg();
}

enum class LLVMCrabLangDiagnosticLevel {
    Error,
    Warning,
    Note,
    Remark,
};

extern "C" void
LLVMCrabLangUnpackInlineAsmDiagnostic(LLVMDiagnosticInfoRef DI,
                                  LLVMCrabLangDiagnosticLevel *LevelOut,
                                  unsigned *CookieOut,
                                  LLVMTwineRef *MessageOut) {
  // Undefined to call this not on an inline assembly diagnostic!
  llvm::DiagnosticInfoInlineAsm *IA =
      static_cast<llvm::DiagnosticInfoInlineAsm *>(unwrap(DI));

  *CookieOut = IA->getLocCookie();
  *MessageOut = wrap(&IA->getMsgStr());

  switch (IA->getSeverity()) {
    case DS_Error:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Error;
      break;
    case DS_Warning:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Warning;
      break;
    case DS_Note:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Note;
      break;
    case DS_Remark:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Remark;
      break;
    default:
      report_fatal_error("Invalid LLVMCrabLangDiagnosticLevel value!");
  }
}

extern "C" void LLVMCrabLangWriteDiagnosticInfoToString(LLVMDiagnosticInfoRef DI,
                                                    CrabLangStringRef Str) {
  RawCrabLangStringOstream OS(Str);
  DiagnosticPrinterRawOStream DP(OS);
  unwrap(DI)->print(DP);
}

enum class LLVMCrabLangDiagnosticKind {
  Other,
  InlineAsm,
  StackSize,
  DebugMetadataVersion,
  SampleProfile,
  OptimizationRemark,
  OptimizationRemarkMissed,
  OptimizationRemarkAnalysis,
  OptimizationRemarkAnalysisFPCommute,
  OptimizationRemarkAnalysisAliasing,
  OptimizationRemarkOther,
  OptimizationFailure,
  PGOProfile,
  Linker,
  Unsupported,
  SrcMgr,
};

static LLVMCrabLangDiagnosticKind toCrabLang(DiagnosticKind Kind) {
  switch (Kind) {
  case DK_InlineAsm:
    return LLVMCrabLangDiagnosticKind::InlineAsm;
  case DK_StackSize:
    return LLVMCrabLangDiagnosticKind::StackSize;
  case DK_DebugMetadataVersion:
    return LLVMCrabLangDiagnosticKind::DebugMetadataVersion;
  case DK_SampleProfile:
    return LLVMCrabLangDiagnosticKind::SampleProfile;
  case DK_OptimizationRemark:
  case DK_MachineOptimizationRemark:
    return LLVMCrabLangDiagnosticKind::OptimizationRemark;
  case DK_OptimizationRemarkMissed:
  case DK_MachineOptimizationRemarkMissed:
    return LLVMCrabLangDiagnosticKind::OptimizationRemarkMissed;
  case DK_OptimizationRemarkAnalysis:
  case DK_MachineOptimizationRemarkAnalysis:
    return LLVMCrabLangDiagnosticKind::OptimizationRemarkAnalysis;
  case DK_OptimizationRemarkAnalysisFPCommute:
    return LLVMCrabLangDiagnosticKind::OptimizationRemarkAnalysisFPCommute;
  case DK_OptimizationRemarkAnalysisAliasing:
    return LLVMCrabLangDiagnosticKind::OptimizationRemarkAnalysisAliasing;
  case DK_PGOProfile:
    return LLVMCrabLangDiagnosticKind::PGOProfile;
  case DK_Linker:
    return LLVMCrabLangDiagnosticKind::Linker;
  case DK_Unsupported:
    return LLVMCrabLangDiagnosticKind::Unsupported;
  case DK_SrcMgr:
    return LLVMCrabLangDiagnosticKind::SrcMgr;
  default:
    return (Kind >= DK_FirstRemark && Kind <= DK_LastRemark)
               ? LLVMCrabLangDiagnosticKind::OptimizationRemarkOther
               : LLVMCrabLangDiagnosticKind::Other;
  }
}

extern "C" LLVMCrabLangDiagnosticKind
LLVMCrabLangGetDiagInfoKind(LLVMDiagnosticInfoRef DI) {
  return toCrabLang((DiagnosticKind)unwrap(DI)->getKind());
}

// This is kept distinct from LLVMGetTypeKind, because when
// a new type kind is added, the CrabLang-side enum must be
// updated or UB will result.
extern "C" LLVMTypeKind LLVMCrabLangGetTypeKind(LLVMTypeRef Ty) {
  switch (unwrap(Ty)->getTypeID()) {
  case Type::VoidTyID:
    return LLVMVoidTypeKind;
  case Type::HalfTyID:
    return LLVMHalfTypeKind;
  case Type::FloatTyID:
    return LLVMFloatTypeKind;
  case Type::DoubleTyID:
    return LLVMDoubleTypeKind;
  case Type::X86_FP80TyID:
    return LLVMX86_FP80TypeKind;
  case Type::FP128TyID:
    return LLVMFP128TypeKind;
  case Type::PPC_FP128TyID:
    return LLVMPPC_FP128TypeKind;
  case Type::LabelTyID:
    return LLVMLabelTypeKind;
  case Type::MetadataTyID:
    return LLVMMetadataTypeKind;
  case Type::IntegerTyID:
    return LLVMIntegerTypeKind;
  case Type::FunctionTyID:
    return LLVMFunctionTypeKind;
  case Type::StructTyID:
    return LLVMStructTypeKind;
  case Type::ArrayTyID:
    return LLVMArrayTypeKind;
  case Type::PointerTyID:
    return LLVMPointerTypeKind;
  case Type::FixedVectorTyID:
    return LLVMVectorTypeKind;
  case Type::X86_MMXTyID:
    return LLVMX86_MMXTypeKind;
  case Type::TokenTyID:
    return LLVMTokenTypeKind;
  case Type::ScalableVectorTyID:
    return LLVMScalableVectorTypeKind;
  case Type::BFloatTyID:
    return LLVMBFloatTypeKind;
  case Type::X86_AMXTyID:
    return LLVMX86_AMXTypeKind;
  default:
    {
      std::string error;
      llvm::raw_string_ostream stream(error);
      stream << "CrabLang does not support the TypeID: " << unwrap(Ty)->getTypeID()
             << " for the type: " << *unwrap(Ty);
      stream.flush();
      report_fatal_error(error.c_str());
    }
  }
}

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(SMDiagnostic, LLVMSMDiagnosticRef)

extern "C" LLVMSMDiagnosticRef LLVMCrabLangGetSMDiagnostic(
    LLVMDiagnosticInfoRef DI, unsigned *Cookie) {
  llvm::DiagnosticInfoSrcMgr *SM = static_cast<llvm::DiagnosticInfoSrcMgr *>(unwrap(DI));
  *Cookie = SM->getLocCookie();
  return wrap(&SM->getSMDiag());
}

extern "C" bool LLVMCrabLangUnpackSMDiagnostic(LLVMSMDiagnosticRef DRef,
                                           CrabLangStringRef MessageOut,
                                           CrabLangStringRef BufferOut,
                                           LLVMCrabLangDiagnosticLevel* LevelOut,
                                           unsigned* LocOut,
                                           unsigned* RangesOut,
                                           size_t* NumRanges) {
  SMDiagnostic& D = *unwrap(DRef);
  RawCrabLangStringOstream MessageOS(MessageOut);
  MessageOS << D.getMessage();

  switch (D.getKind()) {
    case SourceMgr::DK_Error:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Error;
      break;
    case SourceMgr::DK_Warning:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Warning;
      break;
    case SourceMgr::DK_Note:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Note;
      break;
    case SourceMgr::DK_Remark:
      *LevelOut = LLVMCrabLangDiagnosticLevel::Remark;
      break;
    default:
      report_fatal_error("Invalid LLVMCrabLangDiagnosticLevel value!");
  }

  if (D.getLoc() == SMLoc())
    return false;

  const SourceMgr &LSM = *D.getSourceMgr();
  const MemoryBuffer *LBuf = LSM.getMemoryBuffer(LSM.FindBufferContainingLoc(D.getLoc()));
  LLVMCrabLangStringWriteImpl(BufferOut, LBuf->getBufferStart(), LBuf->getBufferSize());

  *LocOut = D.getLoc().getPointer() - LBuf->getBufferStart();

  *NumRanges = std::min(*NumRanges, D.getRanges().size());
  size_t LineStart = *LocOut - (size_t)D.getColumnNo();
  for (size_t i = 0; i < *NumRanges; i++) {
    RangesOut[i * 2] = LineStart + D.getRanges()[i].first;
    RangesOut[i * 2 + 1] = LineStart + D.getRanges()[i].second;
  }

  return true;
}

extern "C" OperandBundleDef *LLVMCrabLangBuildOperandBundleDef(const char *Name,
                                                           LLVMValueRef *Inputs,
                                                           unsigned NumInputs) {
  return new OperandBundleDef(Name, ArrayRef<Value*>(unwrap(Inputs), NumInputs));
}

extern "C" void LLVMCrabLangFreeOperandBundleDef(OperandBundleDef *Bundle) {
  delete Bundle;
}

extern "C" LLVMValueRef LLVMCrabLangBuildCall(LLVMBuilderRef B, LLVMTypeRef Ty, LLVMValueRef Fn,
                                          LLVMValueRef *Args, unsigned NumArgs,
                                          OperandBundleDef **OpBundles,
                                          unsigned NumOpBundles) {
  Value *Callee = unwrap(Fn);
  FunctionType *FTy = unwrap<FunctionType>(Ty);
  return wrap(unwrap(B)->CreateCall(
      FTy, Callee, ArrayRef<Value*>(unwrap(Args), NumArgs),
      ArrayRef<OperandBundleDef>(*OpBundles, NumOpBundles)));
}

extern "C" LLVMValueRef LLVMCrabLangGetInstrProfIncrementIntrinsic(LLVMModuleRef M) {
  return wrap(llvm::Intrinsic::getDeclaration(unwrap(M),
              (llvm::Intrinsic::ID)llvm::Intrinsic::instrprof_increment));
}

extern "C" LLVMValueRef LLVMCrabLangBuildMemCpy(LLVMBuilderRef B,
                                            LLVMValueRef Dst, unsigned DstAlign,
                                            LLVMValueRef Src, unsigned SrcAlign,
                                            LLVMValueRef Size, bool IsVolatile) {
  return wrap(unwrap(B)->CreateMemCpy(
      unwrap(Dst), MaybeAlign(DstAlign),
      unwrap(Src), MaybeAlign(SrcAlign),
      unwrap(Size), IsVolatile));
}

extern "C" LLVMValueRef LLVMCrabLangBuildMemMove(LLVMBuilderRef B,
                                             LLVMValueRef Dst, unsigned DstAlign,
                                             LLVMValueRef Src, unsigned SrcAlign,
                                             LLVMValueRef Size, bool IsVolatile) {
  return wrap(unwrap(B)->CreateMemMove(
      unwrap(Dst), MaybeAlign(DstAlign),
      unwrap(Src), MaybeAlign(SrcAlign),
      unwrap(Size), IsVolatile));
}

extern "C" LLVMValueRef LLVMCrabLangBuildMemSet(LLVMBuilderRef B,
                                            LLVMValueRef Dst, unsigned DstAlign,
                                            LLVMValueRef Val,
                                            LLVMValueRef Size, bool IsVolatile) {
  return wrap(unwrap(B)->CreateMemSet(
      unwrap(Dst), unwrap(Val), unwrap(Size), MaybeAlign(DstAlign), IsVolatile));
}

extern "C" LLVMValueRef
LLVMCrabLangBuildInvoke(LLVMBuilderRef B, LLVMTypeRef Ty, LLVMValueRef Fn,
                    LLVMValueRef *Args, unsigned NumArgs,
                    LLVMBasicBlockRef Then, LLVMBasicBlockRef Catch,
                    OperandBundleDef **OpBundles, unsigned NumOpBundles,
                    const char *Name) {
  Value *Callee = unwrap(Fn);
  FunctionType *FTy = unwrap<FunctionType>(Ty);
  return wrap(unwrap(B)->CreateInvoke(FTy, Callee, unwrap(Then), unwrap(Catch),
                                      ArrayRef<Value*>(unwrap(Args), NumArgs),
                                      ArrayRef<OperandBundleDef>(*OpBundles, NumOpBundles),
                                      Name));
}

extern "C" void LLVMCrabLangPositionBuilderAtStart(LLVMBuilderRef B,
                                               LLVMBasicBlockRef BB) {
  auto Point = unwrap(BB)->getFirstInsertionPt();
  unwrap(B)->SetInsertPoint(unwrap(BB), Point);
}

extern "C" void LLVMCrabLangSetComdat(LLVMModuleRef M, LLVMValueRef V,
                                  const char *Name, size_t NameLen) {
  Triple TargetTriple(unwrap(M)->getTargetTriple());
  GlobalObject *GV = unwrap<GlobalObject>(V);
  if (TargetTriple.supportsCOMDAT()) {
    StringRef NameRef(Name, NameLen);
    GV->setComdat(unwrap(M)->getOrInsertComdat(NameRef));
  }
}

enum class LLVMCrabLangLinkage {
  ExternalLinkage = 0,
  AvailableExternallyLinkage = 1,
  LinkOnceAnyLinkage = 2,
  LinkOnceODRLinkage = 3,
  WeakAnyLinkage = 4,
  WeakODRLinkage = 5,
  AppendingLinkage = 6,
  InternalLinkage = 7,
  PrivateLinkage = 8,
  ExternalWeakLinkage = 9,
  CommonLinkage = 10,
};

static LLVMCrabLangLinkage toCrabLang(LLVMLinkage Linkage) {
  switch (Linkage) {
  case LLVMExternalLinkage:
    return LLVMCrabLangLinkage::ExternalLinkage;
  case LLVMAvailableExternallyLinkage:
    return LLVMCrabLangLinkage::AvailableExternallyLinkage;
  case LLVMLinkOnceAnyLinkage:
    return LLVMCrabLangLinkage::LinkOnceAnyLinkage;
  case LLVMLinkOnceODRLinkage:
    return LLVMCrabLangLinkage::LinkOnceODRLinkage;
  case LLVMWeakAnyLinkage:
    return LLVMCrabLangLinkage::WeakAnyLinkage;
  case LLVMWeakODRLinkage:
    return LLVMCrabLangLinkage::WeakODRLinkage;
  case LLVMAppendingLinkage:
    return LLVMCrabLangLinkage::AppendingLinkage;
  case LLVMInternalLinkage:
    return LLVMCrabLangLinkage::InternalLinkage;
  case LLVMPrivateLinkage:
    return LLVMCrabLangLinkage::PrivateLinkage;
  case LLVMExternalWeakLinkage:
    return LLVMCrabLangLinkage::ExternalWeakLinkage;
  case LLVMCommonLinkage:
    return LLVMCrabLangLinkage::CommonLinkage;
  default:
    report_fatal_error("Invalid LLVMCrabLangLinkage value!");
  }
}

static LLVMLinkage fromCrabLang(LLVMCrabLangLinkage Linkage) {
  switch (Linkage) {
  case LLVMCrabLangLinkage::ExternalLinkage:
    return LLVMExternalLinkage;
  case LLVMCrabLangLinkage::AvailableExternallyLinkage:
    return LLVMAvailableExternallyLinkage;
  case LLVMCrabLangLinkage::LinkOnceAnyLinkage:
    return LLVMLinkOnceAnyLinkage;
  case LLVMCrabLangLinkage::LinkOnceODRLinkage:
    return LLVMLinkOnceODRLinkage;
  case LLVMCrabLangLinkage::WeakAnyLinkage:
    return LLVMWeakAnyLinkage;
  case LLVMCrabLangLinkage::WeakODRLinkage:
    return LLVMWeakODRLinkage;
  case LLVMCrabLangLinkage::AppendingLinkage:
    return LLVMAppendingLinkage;
  case LLVMCrabLangLinkage::InternalLinkage:
    return LLVMInternalLinkage;
  case LLVMCrabLangLinkage::PrivateLinkage:
    return LLVMPrivateLinkage;
  case LLVMCrabLangLinkage::ExternalWeakLinkage:
    return LLVMExternalWeakLinkage;
  case LLVMCrabLangLinkage::CommonLinkage:
    return LLVMCommonLinkage;
  }
  report_fatal_error("Invalid LLVMCrabLangLinkage value!");
}

extern "C" LLVMCrabLangLinkage LLVMCrabLangGetLinkage(LLVMValueRef V) {
  return toCrabLang(LLVMGetLinkage(V));
}

extern "C" void LLVMCrabLangSetLinkage(LLVMValueRef V,
                                   LLVMCrabLangLinkage CrabLangLinkage) {
  LLVMSetLinkage(V, fromCrabLang(CrabLangLinkage));
}

// FIXME: replace with LLVMConstInBoundsGEP2 when bumped minimal version to llvm-14
extern "C" LLVMValueRef LLVMCrabLangConstInBoundsGEP2(LLVMTypeRef Ty,
                                                  LLVMValueRef ConstantVal,
                                                  LLVMValueRef *ConstantIndices,
                                                  unsigned NumIndices) {
  ArrayRef<Constant *> IdxList(unwrap<Constant>(ConstantIndices, NumIndices),
                               NumIndices);
  Constant *Val = unwrap<Constant>(ConstantVal);
  return wrap(ConstantExpr::getInBoundsGetElementPtr(unwrap(Ty), Val, IdxList));
}

extern "C" bool LLVMCrabLangConstIntGetZExtValue(LLVMValueRef CV, uint64_t *value) {
    auto C = unwrap<llvm::ConstantInt>(CV);
    if (C->getBitWidth() > 64)
      return false;
    *value = C->getZExtValue();
    return true;
}

// Returns true if both high and low were successfully set. Fails in case constant wasn’t any of
// the common sizes (1, 8, 16, 32, 64, 128 bits)
extern "C" bool LLVMCrabLangConstInt128Get(LLVMValueRef CV, bool sext, uint64_t *high, uint64_t *low)
{
    auto C = unwrap<llvm::ConstantInt>(CV);
    if (C->getBitWidth() > 128) { return false; }
    APInt AP;
#if LLVM_VERSION_GE(15, 0)
    if (sext) {
        AP = C->getValue().sext(128);
    } else {
        AP = C->getValue().zext(128);
    }
#else
    if (sext) {
        AP = C->getValue().sextOrSelf(128);
    } else {
        AP = C->getValue().zextOrSelf(128);
    }
#endif
    *low = AP.getLoBits(64).getZExtValue();
    *high = AP.getHiBits(64).getZExtValue();
    return true;
}

enum class LLVMCrabLangVisibility {
  Default = 0,
  Hidden = 1,
  Protected = 2,
};

static LLVMCrabLangVisibility toCrabLang(LLVMVisibility Vis) {
  switch (Vis) {
  case LLVMDefaultVisibility:
    return LLVMCrabLangVisibility::Default;
  case LLVMHiddenVisibility:
    return LLVMCrabLangVisibility::Hidden;
  case LLVMProtectedVisibility:
    return LLVMCrabLangVisibility::Protected;
  }
  report_fatal_error("Invalid LLVMCrabLangVisibility value!");
}

static LLVMVisibility fromCrabLang(LLVMCrabLangVisibility Vis) {
  switch (Vis) {
  case LLVMCrabLangVisibility::Default:
    return LLVMDefaultVisibility;
  case LLVMCrabLangVisibility::Hidden:
    return LLVMHiddenVisibility;
  case LLVMCrabLangVisibility::Protected:
    return LLVMProtectedVisibility;
  }
  report_fatal_error("Invalid LLVMCrabLangVisibility value!");
}

extern "C" LLVMCrabLangVisibility LLVMCrabLangGetVisibility(LLVMValueRef V) {
  return toCrabLang(LLVMGetVisibility(V));
}

extern "C" void LLVMCrabLangSetVisibility(LLVMValueRef V,
                                      LLVMCrabLangVisibility CrabLangVisibility) {
  LLVMSetVisibility(V, fromCrabLang(CrabLangVisibility));
}

extern "C" void LLVMCrabLangSetDSOLocal(LLVMValueRef Global, bool is_dso_local) {
  unwrap<GlobalValue>(Global)->setDSOLocal(is_dso_local);
}

struct LLVMCrabLangModuleBuffer {
  std::string data;
};

extern "C" LLVMCrabLangModuleBuffer*
LLVMCrabLangModuleBufferCreate(LLVMModuleRef M) {
  auto Ret = std::make_unique<LLVMCrabLangModuleBuffer>();
  {
    raw_string_ostream OS(Ret->data);
    WriteBitcodeToFile(*unwrap(M), OS);
  }
  return Ret.release();
}

extern "C" void
LLVMCrabLangModuleBufferFree(LLVMCrabLangModuleBuffer *Buffer) {
  delete Buffer;
}

extern "C" const void*
LLVMCrabLangModuleBufferPtr(const LLVMCrabLangModuleBuffer *Buffer) {
  return Buffer->data.data();
}

extern "C" size_t
LLVMCrabLangModuleBufferLen(const LLVMCrabLangModuleBuffer *Buffer) {
  return Buffer->data.length();
}

extern "C" uint64_t
LLVMCrabLangModuleCost(LLVMModuleRef M) {
  auto f = unwrap(M)->functions();
  return std::distance(std::begin(f), std::end(f));
}

extern "C" void
LLVMCrabLangModuleInstructionStats(LLVMModuleRef M, CrabLangStringRef Str)
{
  RawCrabLangStringOstream OS(Str);
  llvm::json::OStream JOS(OS);
  auto Module = unwrap(M);

  JOS.object([&] {
    JOS.attribute("module", Module->getName());
    JOS.attribute("total", Module->getInstructionCount());
  });
}

// Vector reductions:
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceFAdd(LLVMBuilderRef B, LLVMValueRef Acc, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateFAddReduce(unwrap(Acc),unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceFMul(LLVMBuilderRef B, LLVMValueRef Acc, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateFMulReduce(unwrap(Acc),unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceAdd(LLVMBuilderRef B, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateAddReduce(unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceMul(LLVMBuilderRef B, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateMulReduce(unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceAnd(LLVMBuilderRef B, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateAndReduce(unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceOr(LLVMBuilderRef B, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateOrReduce(unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceXor(LLVMBuilderRef B, LLVMValueRef Src) {
    return wrap(unwrap(B)->CreateXorReduce(unwrap(Src)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceMin(LLVMBuilderRef B, LLVMValueRef Src, bool IsSigned) {
    return wrap(unwrap(B)->CreateIntMinReduce(unwrap(Src), IsSigned));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceMax(LLVMBuilderRef B, LLVMValueRef Src, bool IsSigned) {
    return wrap(unwrap(B)->CreateIntMaxReduce(unwrap(Src), IsSigned));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceFMin(LLVMBuilderRef B, LLVMValueRef Src, bool NoNaN) {
  Instruction *I = unwrap(B)->CreateFPMinReduce(unwrap(Src));
  I->setHasNoNaNs(NoNaN);
  return wrap(I);
}
extern "C" LLVMValueRef
LLVMCrabLangBuildVectorReduceFMax(LLVMBuilderRef B, LLVMValueRef Src, bool NoNaN) {
  Instruction *I = unwrap(B)->CreateFPMaxReduce(unwrap(Src));
  I->setHasNoNaNs(NoNaN);
  return wrap(I);
}

extern "C" LLVMValueRef
LLVMCrabLangBuildMinNum(LLVMBuilderRef B, LLVMValueRef LHS, LLVMValueRef RHS) {
    return wrap(unwrap(B)->CreateMinNum(unwrap(LHS),unwrap(RHS)));
}
extern "C" LLVMValueRef
LLVMCrabLangBuildMaxNum(LLVMBuilderRef B, LLVMValueRef LHS, LLVMValueRef RHS) {
    return wrap(unwrap(B)->CreateMaxNum(unwrap(LHS),unwrap(RHS)));
}

// This struct contains all necessary info about a symbol exported from a DLL.
struct LLVMCrabLangCOFFShortExport {
  const char* name;
  bool ordinal_present;
  // The value of `ordinal` is only meaningful if `ordinal_present` is true.
  uint16_t ordinal;
};

// Machine must be a COFF machine type, as defined in PE specs.
extern "C" LLVMCrabLangResult LLVMCrabLangWriteImportLibrary(
  const char* ImportName,
  const char* Path,
  const LLVMCrabLangCOFFShortExport* Exports,
  size_t NumExports,
  uint16_t Machine,
  bool MinGW)
{
  std::vector<llvm::object::COFFShortExport> ConvertedExports;
  ConvertedExports.reserve(NumExports);

  for (size_t i = 0; i < NumExports; ++i) {
    bool ordinal_present = Exports[i].ordinal_present;
    uint16_t ordinal = ordinal_present ? Exports[i].ordinal : 0;
    ConvertedExports.push_back(llvm::object::COFFShortExport{
      Exports[i].name,  // Name
      std::string{},    // ExtName
      std::string{},    // SymbolName
      std::string{},    // AliasTarget
      ordinal,          // Ordinal
      ordinal_present,  // Noname
      false,            // Data
      false,            // Private
      false             // Constant
    });
  }

  auto Error = llvm::object::writeImportLibrary(
    ImportName,
    Path,
    ConvertedExports,
    static_cast<llvm::COFF::MachineTypes>(Machine),
    MinGW);
  if (Error) {
    std::string errorString;
    llvm::raw_string_ostream stream(errorString);
    stream << Error;
    stream.flush();
    LLVMCrabLangSetLastError(errorString.c_str());
    return LLVMCrabLangResult::Failure;
  } else {
    return LLVMCrabLangResult::Success;
  }
}

// Transfers ownership of DiagnosticHandler unique_ptr to the caller.
extern "C" DiagnosticHandler *
LLVMCrabLangContextGetDiagnosticHandler(LLVMContextRef C) {
  std::unique_ptr<DiagnosticHandler> DH = unwrap(C)->getDiagnosticHandler();
  return DH.release();
}

// Sets unique_ptr to object of DiagnosticHandler to provide custom diagnostic
// handling. Ownership of the handler is moved to the LLVMContext.
extern "C" void LLVMCrabLangContextSetDiagnosticHandler(LLVMContextRef C,
                                                    DiagnosticHandler *DH) {
  unwrap(C)->setDiagnosticHandler(std::unique_ptr<DiagnosticHandler>(DH));
}

using LLVMDiagnosticHandlerTy = DiagnosticHandler::DiagnosticHandlerTy;

// Configures a diagnostic handler that invokes provided callback when a
// backend needs to emit a diagnostic.
//
// When RemarkAllPasses is true, remarks are enabled for all passes. Otherwise
// the RemarkPasses array specifies individual passes for which remarks will be
// enabled.
extern "C" void LLVMCrabLangContextConfigureDiagnosticHandler(
    LLVMContextRef C, LLVMDiagnosticHandlerTy DiagnosticHandlerCallback,
    void *DiagnosticHandlerContext, bool RemarkAllPasses,
    const char * const * RemarkPasses, size_t RemarkPassesLen) {

  class CrabLangDiagnosticHandler final : public DiagnosticHandler {
  public:
    CrabLangDiagnosticHandler(LLVMDiagnosticHandlerTy DiagnosticHandlerCallback,
                          void *DiagnosticHandlerContext,
                          bool RemarkAllPasses,
                          std::vector<std::string> RemarkPasses)
        : DiagnosticHandlerCallback(DiagnosticHandlerCallback),
          DiagnosticHandlerContext(DiagnosticHandlerContext),
          RemarkAllPasses(RemarkAllPasses),
          RemarkPasses(RemarkPasses) {}

    virtual bool handleDiagnostics(const DiagnosticInfo &DI) override {
      if (DiagnosticHandlerCallback) {
        DiagnosticHandlerCallback(DI, DiagnosticHandlerContext);
        return true;
      }
      return false;
    }

    bool isAnalysisRemarkEnabled(StringRef PassName) const override {
      return isRemarkEnabled(PassName);
    }

    bool isMissedOptRemarkEnabled(StringRef PassName) const override {
      return isRemarkEnabled(PassName);
    }

    bool isPassedOptRemarkEnabled(StringRef PassName) const override {
      return isRemarkEnabled(PassName);
    }

    bool isAnyRemarkEnabled() const override {
      return RemarkAllPasses || !RemarkPasses.empty();
    }

  private:
    bool isRemarkEnabled(StringRef PassName) const {
      if (RemarkAllPasses)
        return true;

      for (auto &Pass : RemarkPasses)
        if (Pass == PassName)
          return true;

      return false;
    }

    LLVMDiagnosticHandlerTy DiagnosticHandlerCallback = nullptr;
    void *DiagnosticHandlerContext = nullptr;

    bool RemarkAllPasses = false;
    std::vector<std::string> RemarkPasses;
  };

  std::vector<std::string> Passes;
  for (size_t I = 0; I != RemarkPassesLen; ++I)
    Passes.push_back(RemarkPasses[I]);

  unwrap(C)->setDiagnosticHandler(std::make_unique<CrabLangDiagnosticHandler>(
      DiagnosticHandlerCallback, DiagnosticHandlerContext, RemarkAllPasses, Passes));
}

extern "C" void LLVMCrabLangGetMangledName(LLVMValueRef V, CrabLangStringRef Str) {
  RawCrabLangStringOstream OS(Str);
  GlobalValue *GV = unwrap<GlobalValue>(V);
  Mangler().getNameWithPrefix(OS, GV, true);
}

// LLVMGetAggregateElement was added in LLVM 15. For earlier LLVM versions just
// use its implementation.
#if LLVM_VERSION_LT(15, 0)
extern "C" LLVMValueRef LLVMGetAggregateElement(LLVMValueRef C, unsigned Idx) {
    return wrap(unwrap<Constant>(C)->getAggregateElement(Idx));
}
#endif

extern "C" int32_t LLVMCrabLangGetElementTypeArgIndex(LLVMValueRef CallSite) {
#if LLVM_VERSION_GE(15, 0)
    auto *CB = unwrap<CallBase>(CallSite);
    switch (CB->getIntrinsicID()) {
        case Intrinsic::arm_ldrex:
            return 0;
        case Intrinsic::arm_strex:
            return 1;
    }
#endif
    return -1;
}

extern "C" bool LLVMCrabLangIsBitcode(char *ptr, size_t len) {
  return identify_magic(StringRef(ptr, len)) == file_magic::bitcode;
}
