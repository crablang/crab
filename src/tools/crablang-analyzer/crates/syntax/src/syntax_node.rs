//! This module defines Concrete Syntax Tree (CST), used by crablang-analyzer.
//!
//! The CST includes comments and whitespace, provides a single node type,
//! `SyntaxNode`, and a basic traversal API (parent, children, siblings).
//!
//! The *real* implementation is in the (language-agnostic) `rowan` crate, this
//! module just wraps its API.

use rowan::{GreenNodeBuilder, Language};

use crate::{Parse, SyntaxError, SyntaxKind, TextSize};

pub(crate) use rowan::{GreenNode, GreenToken, NodeOrToken};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrabLanguage {}
impl Language for CrabLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::from(raw.0)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<CrabLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<CrabLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<CrabLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<CrabLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<CrabLanguage>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<CrabLanguage>;

#[derive(Default)]
pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> Parse<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        // Disable block validation, see https://github.com/crablang/crablang-analyzer/pull/10357
        #[allow(clippy::overly_complex_bool_expr)]
        if cfg!(debug_assertions) && false {
            let node = SyntaxNode::new_root(green.clone());
            crate::validation::validate_block_structure(&node);
        }
        Parse::new(green, errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        let kind = CrabLanguage::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let kind = CrabLanguage::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: String, text_pos: TextSize) {
        self.errors.push(SyntaxError::new_at_offset(error, text_pos));
    }
}
