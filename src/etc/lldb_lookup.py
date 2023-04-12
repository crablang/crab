import lldb

from lldb_providers import *
from crablang_types import CrabLangType, classify_struct, classify_union


# BACKCOMPAT: crablang 1.35
def is_hashbrown_hashmap(hash_map):
    return len(hash_map.type.fields) == 1


def classify_crablang_type(type):
    type_class = type.GetTypeClass()
    if type_class == lldb.eTypeClassStruct:
        return classify_struct(type.name, type.fields)
    if type_class == lldb.eTypeClassUnion:
        return classify_union(type.fields)

    return CrabLangType.OTHER


def summary_lookup(valobj, dict):
    # type: (SBValue, dict) -> str
    """Returns the summary provider for the given value"""
    crablang_type = classify_crablang_type(valobj.GetType())

    if crablang_type == CrabLangType.STD_STRING:
        return StdStringSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_OS_STRING:
        return StdOsStringSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_STR:
        return StdStrSummaryProvider(valobj, dict)

    if crablang_type == CrabLangType.STD_VEC:
        return SizeSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_VEC_DEQUE:
        return SizeSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_SLICE:
        return SizeSummaryProvider(valobj, dict)

    if crablang_type == CrabLangType.STD_HASH_MAP:
        return SizeSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_HASH_SET:
        return SizeSummaryProvider(valobj, dict)

    if crablang_type == CrabLangType.STD_RC:
        return StdRcSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_ARC:
        return StdRcSummaryProvider(valobj, dict)

    if crablang_type == CrabLangType.STD_REF:
        return StdRefSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_REF_MUT:
        return StdRefSummaryProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_REF_CELL:
        return StdRefSummaryProvider(valobj, dict)

    if crablang_type == CrabLangType.STD_NONZERO_NUMBER:
        return StdNonZeroNumberSummaryProvider(valobj, dict)

    return ""


def synthetic_lookup(valobj, dict):
    # type: (SBValue, dict) -> object
    """Returns the synthetic provider for the given value"""
    crablang_type = classify_crablang_type(valobj.GetType())

    if crablang_type == CrabLangType.STRUCT:
        return StructSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STRUCT_VARIANT:
        return StructSyntheticProvider(valobj, dict, is_variant=True)
    if crablang_type == CrabLangType.TUPLE:
        return TupleSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.TUPLE_VARIANT:
        return TupleSyntheticProvider(valobj, dict, is_variant=True)
    if crablang_type == CrabLangType.EMPTY:
        return EmptySyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.REGULAR_ENUM:
        discriminant = valobj.GetChildAtIndex(0).GetChildAtIndex(0).GetValueAsUnsigned()
        return synthetic_lookup(valobj.GetChildAtIndex(discriminant), dict)
    if crablang_type == CrabLangType.SINGLETON_ENUM:
        return synthetic_lookup(valobj.GetChildAtIndex(0), dict)

    if crablang_type == CrabLangType.STD_VEC:
        return StdVecSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_VEC_DEQUE:
        return StdVecDequeSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_SLICE:
        return StdSliceSyntheticProvider(valobj, dict)

    if crablang_type == CrabLangType.STD_HASH_MAP:
        if is_hashbrown_hashmap(valobj):
            return StdHashMapSyntheticProvider(valobj, dict)
        else:
            return StdOldHashMapSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_HASH_SET:
        hash_map = valobj.GetChildAtIndex(0)
        if is_hashbrown_hashmap(hash_map):
            return StdHashMapSyntheticProvider(valobj, dict, show_values=False)
        else:
            return StdOldHashMapSyntheticProvider(hash_map, dict, show_values=False)

    if crablang_type == CrabLangType.STD_RC:
        return StdRcSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_ARC:
        return StdRcSyntheticProvider(valobj, dict, is_atomic=True)

    if crablang_type == CrabLangType.STD_CELL:
        return StdCellSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_REF:
        return StdRefSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_REF_MUT:
        return StdRefSyntheticProvider(valobj, dict)
    if crablang_type == CrabLangType.STD_REF_CELL:
        return StdRefSyntheticProvider(valobj, dict, is_cell=True)

    return DefaultSynthteticProvider(valobj, dict)
