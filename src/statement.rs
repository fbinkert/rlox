use crate::expression::Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    IfStmt {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    PrintStmt(Expr),
    Block(Vec<Self>),
    VarDecl {
        name: String,
        initializer: Expr,
    },
}
