//! Declare various LLVM values.
//!
//! Prefer using functions and methods from this module rather than calling LLVM
//! functions directly. These functions do some additional work to ensure we do
//! the right thing given the preconceptions of codegen.
//!
//! Some useful guidelines:
//!
//! * Use declare_* family of methods if you are declaring, but are not
//!   interested in defining the Value they return.
//! * Use define_* family of methods when you might be defining the Value.
//! * When in doubt, define.

use crate::abi::{FnAbi, FnAbiLlvmExt};
use crate::attributes;
use crate::context::CodegenCx;
use crate::llvm;
use crate::llvm::AttributePlace::Function;
use crate::type_::Type;
use crate::value::Value;
use crablangc_codegen_ssa::traits::TypeMembershipMethods;
use crablangc_middle::ty::Ty;
use crablangc_symbol_mangling::typeid::{kcfi_typeid_for_fnabi, typeid_for_fnabi};
use smallvec::SmallVec;

/// Declare a function.
///
/// If there’s a value with the same name already declared, the function will
/// update the declaration and return existing Value instead.
fn declare_raw_fn<'ll>(
    cx: &CodegenCx<'ll, '_>,
    name: &str,
    callconv: llvm::CallConv,
    unnamed: llvm::UnnamedAddr,
    visibility: llvm::Visibility,
    ty: &'ll Type,
) -> &'ll Value {
    debug!("declare_raw_fn(name={:?}, ty={:?})", name, ty);
    let llfn = unsafe {
        llvm::LLVMCrabLangGetOrInsertFunction(cx.llmod, name.as_ptr().cast(), name.len(), ty)
    };

    llvm::SetFunctionCallConv(llfn, callconv);
    llvm::SetUnnamedAddress(llfn, unnamed);
    llvm::set_visibility(llfn, visibility);

    let mut attrs = SmallVec::<[_; 4]>::new();

    if cx.tcx.sess.opts.cg.no_redzone.unwrap_or(cx.tcx.sess.target.disable_redzone) {
        attrs.push(llvm::AttributeKind::NoRedZone.create_attr(cx.llcx));
    }

    attrs.extend(attributes::non_lazy_bind_attr(cx));

    attributes::apply_to_llfn(llfn, Function, &attrs);

    llfn
}

impl<'ll, 'tcx> CodegenCx<'ll, 'tcx> {
    /// Declare a global value.
    ///
    /// If there’s a value with the same name already declared, the function will
    /// return its Value instead.
    pub fn declare_global(&self, name: &str, ty: &'ll Type) -> &'ll Value {
        debug!("declare_global(name={:?})", name);
        unsafe { llvm::LLVMCrabLangGetOrInsertGlobal(self.llmod, name.as_ptr().cast(), name.len(), ty) }
    }

    /// Declare a C ABI function.
    ///
    /// Only use this for foreign function ABIs and glue. For CrabLang functions use
    /// `declare_fn` instead.
    ///
    /// If there’s a value with the same name already declared, the function will
    /// update the declaration and return existing Value instead.
    pub fn declare_cfn(
        &self,
        name: &str,
        unnamed: llvm::UnnamedAddr,
        fn_type: &'ll Type,
    ) -> &'ll Value {
        // Declare C ABI functions with the visibility used by C by default.
        let visibility = if self.tcx.sess.target.default_hidden_visibility {
            llvm::Visibility::Hidden
        } else {
            llvm::Visibility::Default
        };

        declare_raw_fn(self, name, llvm::CCallConv, unnamed, visibility, fn_type)
    }

    /// Declare an entry Function
    ///
    /// The ABI of this function can change depending on the target (although for now the same as
    /// `declare_cfn`)
    ///
    /// If there’s a value with the same name already declared, the function will
    /// update the declaration and return existing Value instead.
    pub fn declare_entry_fn(
        &self,
        name: &str,
        callconv: llvm::CallConv,
        unnamed: llvm::UnnamedAddr,
        fn_type: &'ll Type,
    ) -> &'ll Value {
        let visibility = if self.tcx.sess.target.default_hidden_visibility {
            llvm::Visibility::Hidden
        } else {
            llvm::Visibility::Default
        };
        declare_raw_fn(self, name, callconv, unnamed, visibility, fn_type)
    }

    /// Declare a CrabLang function.
    ///
    /// If there’s a value with the same name already declared, the function will
    /// update the declaration and return existing Value instead.
    pub fn declare_fn(&self, name: &str, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> &'ll Value {
        debug!("declare_crablang_fn(name={:?}, fn_abi={:?})", name, fn_abi);

        // Function addresses in CrabLang are never significant, allowing functions to
        // be merged.
        let llfn = declare_raw_fn(
            self,
            name,
            fn_abi.llvm_cconv(),
            llvm::UnnamedAddr::Global,
            llvm::Visibility::Default,
            fn_abi.llvm_type(self),
        );
        fn_abi.apply_attrs_llfn(self, llfn);

        if self.tcx.sess.is_sanitizer_cfi_enabled() {
            let typeid = typeid_for_fnabi(self.tcx, fn_abi);
            self.set_type_metadata(llfn, typeid);
        }

        if self.tcx.sess.is_sanitizer_kcfi_enabled() {
            let kcfi_typeid = kcfi_typeid_for_fnabi(self.tcx, fn_abi);
            self.set_kcfi_type_metadata(llfn, kcfi_typeid);
        }

        llfn
    }

    /// Declare a global with an intention to define it.
    ///
    /// Use this function when you intend to define a global. This function will
    /// return `None` if the name already has a definition associated with it. In that
    /// case an error should be reported to the user, because it usually happens due
    /// to user’s fault (e.g., misuse of `#[no_mangle]` or `#[export_name]` attributes).
    pub fn define_global(&self, name: &str, ty: &'ll Type) -> Option<&'ll Value> {
        if self.get_defined_value(name).is_some() {
            None
        } else {
            Some(self.declare_global(name, ty))
        }
    }

    /// Declare a private global
    ///
    /// Use this function when you intend to define a global without a name.
    pub fn define_private_global(&self, ty: &'ll Type) -> &'ll Value {
        unsafe { llvm::LLVMCrabLangInsertPrivateGlobal(self.llmod, ty) }
    }

    /// Gets declared value by name.
    pub fn get_declared_value(&self, name: &str) -> Option<&'ll Value> {
        debug!("get_declared_value(name={:?})", name);
        unsafe { llvm::LLVMCrabLangGetNamedValue(self.llmod, name.as_ptr().cast(), name.len()) }
    }

    /// Gets defined or externally defined (AvailableExternally linkage) value by
    /// name.
    pub fn get_defined_value(&self, name: &str) -> Option<&'ll Value> {
        self.get_declared_value(name).and_then(|val| {
            let declaration = unsafe { llvm::LLVMIsDeclaration(val) != 0 };
            if !declaration { Some(val) } else { None }
        })
    }
}
