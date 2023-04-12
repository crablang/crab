#include "LLVMWrapper.h"
#include "llvm/ProfileData/Coverage/CoverageMapping.h"
#include "llvm/ProfileData/Coverage/CoverageMappingWriter.h"
#include "llvm/ProfileData/InstrProf.h"
#include "llvm/ADT/ArrayRef.h"

#include <iostream>

using namespace llvm;

struct LLVMCrabLangCounterMappingRegion {
  coverage::Counter Count;
  coverage::Counter FalseCount;
  uint32_t FileID;
  uint32_t ExpandedFileID;
  uint32_t LineStart;
  uint32_t ColumnStart;
  uint32_t LineEnd;
  uint32_t ColumnEnd;
  coverage::CounterMappingRegion::RegionKind Kind;
};

extern "C" void LLVMCrabLangCoverageWriteFilenamesSectionToBuffer(
    const char* const Filenames[],
    size_t FilenamesLen,
    CrabLangStringRef BufferOut) {
  SmallVector<std::string,32> FilenameRefs;
  for (size_t i = 0; i < FilenamesLen; i++) {
    FilenameRefs.push_back(std::string(Filenames[i]));
  }
  auto FilenamesWriter =
      coverage::CoverageFilenamesSectionWriter(ArrayRef<std::string>(FilenameRefs));
  RawCrabLangStringOstream OS(BufferOut);
  FilenamesWriter.write(OS);
}

extern "C" void LLVMCrabLangCoverageWriteMappingToBuffer(
    const unsigned *VirtualFileMappingIDs,
    unsigned NumVirtualFileMappingIDs,
    const coverage::CounterExpression *Expressions,
    unsigned NumExpressions,
    LLVMCrabLangCounterMappingRegion *CrabLangMappingRegions,
    unsigned NumMappingRegions,
    CrabLangStringRef BufferOut) {
  // Convert from FFI representation to LLVM representation.
  SmallVector<coverage::CounterMappingRegion, 0> MappingRegions;
  MappingRegions.reserve(NumMappingRegions);
  for (const auto &Region : ArrayRef<LLVMCrabLangCounterMappingRegion>(
           CrabLangMappingRegions, NumMappingRegions)) {
    MappingRegions.emplace_back(
        Region.Count, Region.FalseCount, Region.FileID, Region.ExpandedFileID,
        Region.LineStart, Region.ColumnStart, Region.LineEnd, Region.ColumnEnd,
        Region.Kind);
  }
  auto CoverageMappingWriter = coverage::CoverageMappingWriter(
      ArrayRef<unsigned>(VirtualFileMappingIDs, NumVirtualFileMappingIDs),
      ArrayRef<coverage::CounterExpression>(Expressions, NumExpressions),
      MappingRegions);
  RawCrabLangStringOstream OS(BufferOut);
  CoverageMappingWriter.write(OS);
}

extern "C" LLVMValueRef LLVMCrabLangCoverageCreatePGOFuncNameVar(LLVMValueRef F, const char *FuncName) {
  StringRef FuncNameRef(FuncName);
  return wrap(createPGOFuncNameVar(*cast<Function>(unwrap(F)), FuncNameRef));
}

extern "C" uint64_t LLVMCrabLangCoverageHashCString(const char *StrVal) {
  StringRef StrRef(StrVal);
  return IndexedInstrProf::ComputeHash(StrRef);
}

extern "C" uint64_t LLVMCrabLangCoverageHashByteArray(
    const char *Bytes,
    unsigned NumBytes) {
  StringRef StrRef(Bytes, NumBytes);
  return IndexedInstrProf::ComputeHash(StrRef);
}

static void WriteSectionNameToString(LLVMModuleRef M,
                                     InstrProfSectKind SK,
                                     CrabLangStringRef Str) {
  Triple TargetTriple(unwrap(M)->getTargetTriple());
  auto name = getInstrProfSectionName(SK, TargetTriple.getObjectFormat());
  RawCrabLangStringOstream OS(Str);
  OS << name;
}

extern "C" void LLVMCrabLangCoverageWriteMapSectionNameToString(LLVMModuleRef M,
                                                            CrabLangStringRef Str) {
  WriteSectionNameToString(M, IPSK_covmap, Str);
}

extern "C" void LLVMCrabLangCoverageWriteFuncSectionNameToString(LLVMModuleRef M,
                                                             CrabLangStringRef Str) {
  WriteSectionNameToString(M, IPSK_covfun, Str);
}

extern "C" void LLVMCrabLangCoverageWriteMappingVarNameToString(CrabLangStringRef Str) {
  auto name = getCoverageMappingVarName();
  RawCrabLangStringOstream OS(Str);
  OS << name;
}

extern "C" uint32_t LLVMCrabLangCoverageMappingVersion() {
  return coverage::CovMapVersion::Version6;
}
