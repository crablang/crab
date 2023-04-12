import re


class CrabLangType(object):
    OTHER = "Other"
    STRUCT = "Struct"
    TUPLE = "Tuple"
    CSTYLE_VARIANT = "CStyleVariant"
    TUPLE_VARIANT = "TupleVariant"
    STRUCT_VARIANT = "StructVariant"
    ENUM = "Enum"
    EMPTY = "Empty"
    SINGLETON_ENUM = "SingletonEnum"
    REGULAR_ENUM = "RegularEnum"
    COMPRESSED_ENUM = "CompressedEnum"
    REGULAR_UNION = "RegularUnion"

    STD_STRING = "StdString"
    STD_OS_STRING = "StdOsString"
    STD_STR = "StdStr"
    STD_SLICE = "StdSlice"
    STD_VEC = "StdVec"
    STD_VEC_DEQUE = "StdVecDeque"
    STD_BTREE_SET = "StdBTreeSet"
    STD_BTREE_MAP = "StdBTreeMap"
    STD_HASH_MAP = "StdHashMap"
    STD_HASH_SET = "StdHashSet"
    STD_RC = "StdRc"
    STD_ARC = "StdArc"
    STD_CELL = "StdCell"
    STD_REF = "StdRef"
    STD_REF_MUT = "StdRefMut"
    STD_REF_CELL = "StdRefCell"
    STD_NONZERO_NUMBER = "StdNonZeroNumber"


STD_STRING_REGEX = re.compile(r"^(alloc::(\w+::)+)String$")
STD_STR_REGEX = re.compile(r"^&(mut )?str$")
STD_SLICE_REGEX = re.compile(r"^&(mut )?\[.+\]$")
STD_OS_STRING_REGEX = re.compile(r"^(std::ffi::(\w+::)+)OsString$")
STD_VEC_REGEX = re.compile(r"^(alloc::(\w+::)+)Vec<.+>$")
STD_VEC_DEQUE_REGEX = re.compile(r"^(alloc::(\w+::)+)VecDeque<.+>$")
STD_BTREE_SET_REGEX = re.compile(r"^(alloc::(\w+::)+)BTreeSet<.+>$")
STD_BTREE_MAP_REGEX = re.compile(r"^(alloc::(\w+::)+)BTreeMap<.+>$")
STD_HASH_MAP_REGEX = re.compile(r"^(std::collections::(\w+::)+)HashMap<.+>$")
STD_HASH_SET_REGEX = re.compile(r"^(std::collections::(\w+::)+)HashSet<.+>$")
STD_RC_REGEX = re.compile(r"^(alloc::(\w+::)+)Rc<.+>$")
STD_ARC_REGEX = re.compile(r"^(alloc::(\w+::)+)Arc<.+>$")
STD_CELL_REGEX = re.compile(r"^(core::(\w+::)+)Cell<.+>$")
STD_REF_REGEX = re.compile(r"^(core::(\w+::)+)Ref<.+>$")
STD_REF_MUT_REGEX = re.compile(r"^(core::(\w+::)+)RefMut<.+>$")
STD_REF_CELL_REGEX = re.compile(r"^(core::(\w+::)+)RefCell<.+>$")
STD_NONZERO_NUMBER_REGEX = re.compile(r"^core::num::([a-z_]+::)*NonZero.+$")

TUPLE_ITEM_REGEX = re.compile(r"__\d+$")

ENCODED_ENUM_PREFIX = "CRABLANG$ENCODED$ENUM$"
ENUM_DISR_FIELD_NAME = "<<variant>>"

STD_TYPE_TO_REGEX = {
    CrabLangType.STD_STRING: STD_STRING_REGEX,
    CrabLangType.STD_OS_STRING: STD_OS_STRING_REGEX,
    CrabLangType.STD_STR: STD_STR_REGEX,
    CrabLangType.STD_SLICE: STD_SLICE_REGEX,
    CrabLangType.STD_VEC: STD_VEC_REGEX,
    CrabLangType.STD_VEC_DEQUE: STD_VEC_DEQUE_REGEX,
    CrabLangType.STD_HASH_MAP: STD_HASH_MAP_REGEX,
    CrabLangType.STD_HASH_SET: STD_HASH_SET_REGEX,
    CrabLangType.STD_BTREE_SET: STD_BTREE_SET_REGEX,
    CrabLangType.STD_BTREE_MAP: STD_BTREE_MAP_REGEX,
    CrabLangType.STD_RC: STD_RC_REGEX,
    CrabLangType.STD_ARC: STD_ARC_REGEX,
    CrabLangType.STD_REF: STD_REF_REGEX,
    CrabLangType.STD_REF_MUT: STD_REF_MUT_REGEX,
    CrabLangType.STD_REF_CELL: STD_REF_CELL_REGEX,
    CrabLangType.STD_CELL: STD_CELL_REGEX,
    CrabLangType.STD_NONZERO_NUMBER: STD_NONZERO_NUMBER_REGEX,
}

def is_tuple_fields(fields):
    # type: (list) -> bool
    return all(TUPLE_ITEM_REGEX.match(str(field.name)) for field in fields)


def classify_struct(name, fields):
    if len(fields) == 0:
        return CrabLangType.EMPTY

    for ty, regex in STD_TYPE_TO_REGEX.items():
        if regex.match(name):
            return ty

    if fields[0].name == ENUM_DISR_FIELD_NAME:
        return CrabLangType.ENUM

    if is_tuple_fields(fields):
        return CrabLangType.TUPLE

    return CrabLangType.STRUCT


def classify_union(fields):
    if len(fields) == 0:
        return CrabLangType.EMPTY

    first_variant_name = fields[0].name
    if first_variant_name is None:
        if len(fields) == 1:
            return CrabLangType.SINGLETON_ENUM
        else:
            return CrabLangType.REGULAR_ENUM
    elif first_variant_name.startswith(ENCODED_ENUM_PREFIX):
        assert len(fields) == 1
        return CrabLangType.COMPRESSED_ENUM
    else:
        return CrabLangType.REGULAR_UNION
