use crate::expression::Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    PrintStmt(Expr),
    Block(Vec<Stmt>),
    VarDecl { name: String, initializer: Expr },
}
