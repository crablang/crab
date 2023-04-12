//! Validity checking for weak lang items

use crate::LangItem;

use crablangc_span::symbol::{sym, Symbol};

macro_rules! weak_lang_items {
    ($($item:ident, $sym:ident;)*) => {
        pub static WEAK_LANG_ITEMS: &[LangItem] = &[$(LangItem::$item,)*];

        impl LangItem {
            pub fn is_weak(self) -> bool {
                matches!(self, $(LangItem::$item)|*)
            }

            pub fn link_name(self) -> Option<Symbol> {
                match self {
                    $( LangItem::$item => Some(sym::$sym),)*
                    _ => None,
                }
            }
        }
    }
}

weak_lang_items! {
    PanicImpl,          crablang_begin_unwind;
    EhPersonality,      crablang_eh_personality;
    EhCatchTypeinfo,    crablang_eh_catch_typeinfo;
}
