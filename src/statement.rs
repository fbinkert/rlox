use crate::expression::Expr;

pub type Program = [Declaration];

#[derive(Debug, Clone)]
pub enum Declaration {
    VarDecl { name: String, initializer: Expr },
    Stmt(Stmt),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    PrintStmt(Expr),
}
