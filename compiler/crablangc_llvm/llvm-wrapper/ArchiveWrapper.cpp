#include "LLVMWrapper.h"

#include "llvm/Object/Archive.h"
#include "llvm/Object/ArchiveWriter.h"
#include "llvm/Support/Path.h"

using namespace llvm;
using namespace llvm::object;

struct CrabLangArchiveMember {
  const char *Filename;
  const char *Name;
  Archive::Child Child;

  CrabLangArchiveMember()
      : Filename(nullptr), Name(nullptr),
        Child(nullptr, nullptr, nullptr)
  {
  }
  ~CrabLangArchiveMember() {}
};

struct CrabLangArchiveIterator {
  bool First;
  Archive::child_iterator Cur;
  Archive::child_iterator End;
  std::unique_ptr<Error> Err;

  CrabLangArchiveIterator(Archive::child_iterator Cur, Archive::child_iterator End,
      std::unique_ptr<Error> Err)
    : First(true),
      Cur(Cur),
      End(End),
      Err(std::move(Err)) {}
};

enum class LLVMCrabLangArchiveKind {
  GNU,
  BSD,
  DARWIN,
  COFF,
};

static Archive::Kind fromCrabLang(LLVMCrabLangArchiveKind Kind) {
  switch (Kind) {
  case LLVMCrabLangArchiveKind::GNU:
    return Archive::K_GNU;
  case LLVMCrabLangArchiveKind::BSD:
    return Archive::K_BSD;
  case LLVMCrabLangArchiveKind::DARWIN:
    return Archive::K_DARWIN;
  case LLVMCrabLangArchiveKind::COFF:
    return Archive::K_COFF;
  default:
    report_fatal_error("Bad ArchiveKind.");
  }
}

typedef OwningBinary<Archive> *LLVMCrabLangArchiveRef;
typedef CrabLangArchiveMember *LLVMCrabLangArchiveMemberRef;
typedef Archive::Child *LLVMCrabLangArchiveChildRef;
typedef Archive::Child const *LLVMCrabLangArchiveChildConstRef;
typedef CrabLangArchiveIterator *LLVMCrabLangArchiveIteratorRef;

extern "C" LLVMCrabLangArchiveRef LLVMCrabLangOpenArchive(char *Path) {
  ErrorOr<std::unique_ptr<MemoryBuffer>> BufOr =
      MemoryBuffer::getFile(Path, -1, false);
  if (!BufOr) {
    LLVMCrabLangSetLastError(BufOr.getError().message().c_str());
    return nullptr;
  }

  Expected<std::unique_ptr<Archive>> ArchiveOr =
      Archive::create(BufOr.get()->getMemBufferRef());

  if (!ArchiveOr) {
    LLVMCrabLangSetLastError(toString(ArchiveOr.takeError()).c_str());
    return nullptr;
  }

  OwningBinary<Archive> *Ret = new OwningBinary<Archive>(
      std::move(ArchiveOr.get()), std::move(BufOr.get()));

  return Ret;
}

extern "C" void LLVMCrabLangDestroyArchive(LLVMCrabLangArchiveRef CrabLangArchive) {
  delete CrabLangArchive;
}

extern "C" LLVMCrabLangArchiveIteratorRef
LLVMCrabLangArchiveIteratorNew(LLVMCrabLangArchiveRef CrabLangArchive) {
  Archive *Archive = CrabLangArchive->getBinary();
  std::unique_ptr<Error> Err = std::make_unique<Error>(Error::success());
  auto Cur = Archive->child_begin(*Err);
  if (*Err) {
    LLVMCrabLangSetLastError(toString(std::move(*Err)).c_str());
    return nullptr;
  }
  auto End = Archive->child_end();
  return new CrabLangArchiveIterator(Cur, End, std::move(Err));
}

extern "C" LLVMCrabLangArchiveChildConstRef
LLVMCrabLangArchiveIteratorNext(LLVMCrabLangArchiveIteratorRef RAI) {
  if (RAI->Cur == RAI->End)
    return nullptr;

  // Advancing the iterator validates the next child, and this can
  // uncover an error. LLVM requires that we check all Errors,
  // so we only advance the iterator if we actually need to fetch
  // the next child.
  // This means we must not advance the iterator in the *first* call,
  // but instead advance it *before* fetching the child in all later calls.
  if (!RAI->First) {
    ++RAI->Cur;
    if (*RAI->Err) {
      LLVMCrabLangSetLastError(toString(std::move(*RAI->Err)).c_str());
      return nullptr;
    }
  } else {
    RAI->First = false;
  }

  if (RAI->Cur == RAI->End)
    return nullptr;

  const Archive::Child &Child = *RAI->Cur.operator->();
  Archive::Child *Ret = new Archive::Child(Child);

  return Ret;
}

extern "C" void LLVMCrabLangArchiveChildFree(LLVMCrabLangArchiveChildRef Child) {
  delete Child;
}

extern "C" void LLVMCrabLangArchiveIteratorFree(LLVMCrabLangArchiveIteratorRef RAI) {
  delete RAI;
}

extern "C" const char *
LLVMCrabLangArchiveChildName(LLVMCrabLangArchiveChildConstRef Child, size_t *Size) {
  Expected<StringRef> NameOrErr = Child->getName();
  if (!NameOrErr) {
    // crablangc_codegen_llvm currently doesn't use this error string, but it might be
    // useful in the future, and in the mean time this tells LLVM that the
    // error was not ignored and that it shouldn't abort the process.
    LLVMCrabLangSetLastError(toString(NameOrErr.takeError()).c_str());
    return nullptr;
  }
  StringRef Name = NameOrErr.get();
  *Size = Name.size();
  return Name.data();
}

extern "C" LLVMCrabLangArchiveMemberRef
LLVMCrabLangArchiveMemberNew(char *Filename, char *Name,
                         LLVMCrabLangArchiveChildRef Child) {
  CrabLangArchiveMember *Member = new CrabLangArchiveMember;
  Member->Filename = Filename;
  Member->Name = Name;
  if (Child)
    Member->Child = *Child;
  return Member;
}

extern "C" void LLVMCrabLangArchiveMemberFree(LLVMCrabLangArchiveMemberRef Member) {
  delete Member;
}

extern "C" LLVMCrabLangResult
LLVMCrabLangWriteArchive(char *Dst, size_t NumMembers,
                     const LLVMCrabLangArchiveMemberRef *NewMembers,
                     bool WriteSymbtab, LLVMCrabLangArchiveKind CrabLangKind) {

  std::vector<NewArchiveMember> Members;
  auto Kind = fromCrabLang(CrabLangKind);

  for (size_t I = 0; I < NumMembers; I++) {
    auto Member = NewMembers[I];
    assert(Member->Name);
    if (Member->Filename) {
      Expected<NewArchiveMember> MOrErr =
          NewArchiveMember::getFile(Member->Filename, true);
      if (!MOrErr) {
        LLVMCrabLangSetLastError(toString(MOrErr.takeError()).c_str());
        return LLVMCrabLangResult::Failure;
      }
      MOrErr->MemberName = sys::path::filename(MOrErr->MemberName);
      Members.push_back(std::move(*MOrErr));
    } else {
      Expected<NewArchiveMember> MOrErr =
          NewArchiveMember::getOldMember(Member->Child, true);
      if (!MOrErr) {
        LLVMCrabLangSetLastError(toString(MOrErr.takeError()).c_str());
        return LLVMCrabLangResult::Failure;
      }
      Members.push_back(std::move(*MOrErr));
    }
  }

  auto Result = writeArchive(Dst, Members, WriteSymbtab, Kind, true, false);
  if (!Result)
    return LLVMCrabLangResult::Success;
  LLVMCrabLangSetLastError(toString(std::move(Result)).c_str());

  return LLVMCrabLangResult::Failure;
}
