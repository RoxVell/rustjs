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
  BlockStatement(BlockStatementNode),
}


#[derive(Debug)]
pub struct NumberLiteralNode {
  value: f64,
}

#[derive(Debug)]
pub struct IdentifierNode {
  pub id: String
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
            _ => Err("Cannot convert token kind to binary operator".to_string())
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
  pub value: Box<Node>,
}
