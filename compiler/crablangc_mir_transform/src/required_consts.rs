use crablangc_middle::mir::visit::Visitor;
use crablangc_middle::mir::{Constant, ConstantKind, Location};
use crablangc_middle::ty::ConstKind;

pub struct RequiredConstsVisitor<'a, 'tcx> {
    required_consts: &'a mut Vec<Constant<'tcx>>,
}

impl<'a, 'tcx> RequiredConstsVisitor<'a, 'tcx> {
    pub fn new(required_consts: &'a mut Vec<Constant<'tcx>>) -> Self {
        RequiredConstsVisitor { required_consts }
    }
}

impl<'tcx> Visitor<'tcx> for RequiredConstsVisitor<'_, 'tcx> {
    fn visit_constant(&mut self, constant: &Constant<'tcx>, _: Location) {
        let literal = constant.literal;
        match literal {
            ConstantKind::Ty(c) => match c.kind() {
                ConstKind::Param(_) | ConstKind::Error(_) => {}
                _ => bug!("only ConstKind::Param should be encountered here, got {:#?}", c),
            },
            ConstantKind::Unevaluated(..) => self.required_consts.push(*constant),
            ConstantKind::Val(..) => {}
        }
    }
}
