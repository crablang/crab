#include "llvm/Linker/Linker.h"

#include "LLVMWrapper.h"

using namespace llvm;

struct CrabLangLinker {
  Linker L;
  LLVMContext &Ctx;

  CrabLangLinker(Module &M) :
    L(M),
    Ctx(M.getContext())
  {}
};

extern "C" CrabLangLinker*
LLVMCrabLangLinkerNew(LLVMModuleRef DstRef) {
  Module *Dst = unwrap(DstRef);

  return new CrabLangLinker(*Dst);
}

extern "C" void
LLVMCrabLangLinkerFree(CrabLangLinker *L) {
  delete L;
}

extern "C" bool
LLVMCrabLangLinkerAdd(CrabLangLinker *L, char *BC, size_t Len) {
  std::unique_ptr<MemoryBuffer> Buf =
      MemoryBuffer::getMemBufferCopy(StringRef(BC, Len));

  Expected<std::unique_ptr<Module>> SrcOrError =
      llvm::getLazyBitcodeModule(Buf->getMemBufferRef(), L->Ctx);
  if (!SrcOrError) {
    LLVMCrabLangSetLastError(toString(SrcOrError.takeError()).c_str());
    return false;
  }

  auto Src = std::move(*SrcOrError);

  if (L->L.linkInModule(std::move(Src))) {
    LLVMCrabLangSetLastError("");
    return false;
  }
  return true;
}
