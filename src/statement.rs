use crate::expression::Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    IfStmt {
        condition: Expr,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    PrintStmt(Expr),
    WhileStmt {
        condition: Expr,
        body: Box<Self>,
    },
    Block(Vec<Self>),
    VarDecl {
        name: String,
        initializer: Expr,
    },
}
