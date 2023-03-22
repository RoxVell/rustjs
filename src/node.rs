use crate::scanner::TokenKind;

#[derive(Debug)]
pub enum Node {
    StringLiteral(String),
    NumberLiteral(f64),
    BooleanLiteral(bool),
    NullLiteral,
    UndefinedLiteral,
    Identifier(IdentifierNode),
    BinaryExpression(BinaryExpressionNode),
    VariableDeclaration(VariableDeclarationNode),
    AssignmentExpression(AssignmentExpressionNode),
    BlockStatement(BlockStatementNode),
    IfStatement(IfStatementNode),
    PrintStatement(PrintStatementNode),
    WhileStatement(WhileStatementNode),
}

#[derive(Debug)]
pub enum AssignmentOperator {
    AddEqual,
    SubEqual,
    DivEqual,
    MulEqual,
    Equal,
}

#[derive(Debug)]
pub struct AssignmentExpressionNode {
    pub left: Box<Node>,
    pub operator: AssignmentOperator,
    pub right: Box<Node>,
}

impl TryFrom<&TokenKind> for AssignmentOperator {
    type Error = String;

    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::PlusEqual => Ok(Self::AddEqual),
            TokenKind::Minus => Ok(Self::SubEqual),
            TokenKind::Mul => Ok(Self::MulEqual),
            TokenKind::Div => Ok(Self::DivEqual),
            TokenKind::Equal => Ok(Self::Equal),
            _ => Err("Cannot convert token kind to assignment operator".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct WhileStatementNode {
    pub condition: Box<Node>,
    pub body: Box<Node>,
}

#[derive(Debug)]
pub struct PrintStatementNode {
    pub expression: Box<Node>
}

#[derive(Debug)]
pub struct IfStatementNode {
    pub condition: Box<Node>,
    pub then_branch: Box<Node>,
    pub else_branch: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct NumberLiteralNode {
    value: f64,
}

#[derive(Debug)]
pub struct IdentifierNode {
    pub id: String,
}

#[derive(Debug)]
pub struct BlockStatementNode {
    pub statements: Vec<Node>,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Div,
    Mul,
}

impl TryFrom<&TokenKind> for BinaryOperator {
    type Error = String;

    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Sub),
            TokenKind::Mul => Ok(Self::Mul),
            TokenKind::Div => Ok(Self::Div),
            _ => Err("Cannot convert token kind to binary operator".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct BinaryExpressionNode {
    pub left: Box<Node>,
    pub operator: BinaryOperator,
    pub right: Box<Node>,
}

#[derive(Debug)]
pub enum VariableDeclarationKind {
    Let,
    Const,
}

#[derive(Debug)]
pub struct VariableDeclarationNode {
    pub kind: VariableDeclarationKind,
    pub id: String,
    pub value: Option<Box<Node>>,
}
