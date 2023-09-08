use crate::scanner::{Span, Token};
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    ProgramStatement(ProgramNode),
    StringLiteral(StringLiteralNode),
    NumberLiteral(f64),
    BooleanLiteral(bool),
    NullLiteral,
    UndefinedLiteral,
    ThisExpression,
    Identifier(IdentifierNode),
    BinaryExpression(BinaryExpressionNode),
    VariableDeclaration(VariableDeclarationNode),
    AssignmentExpression(AssignmentExpressionNode),
    BlockStatement(BlockStatementNode),
    IfStatement(IfStatementNode),
    WhileStatement(WhileStatementNode),
    ForStatement(ForStatementNode),
    FunctionDeclaration(FunctionDeclarationNode),
    FunctionExpression(FunctionExpressionNode),
    ReturnStatement(ReturnStatementNode),
    CallExpression(CallExpressionNode),
    ConditionalExpression(ConditionalExpressionNode),
    MemberExpression(MemberExpressionNode),
    ClassDeclaration(ClassDeclarationNode),
    NewExpression(NewExpressionNode),
    ObjectProperty(ObjectPropertyNode),
    ObjectExpression(ObjectExpressionNode),
    ExpressionStatement(Box<Node>),
}

impl TryFrom<Node> for ObjectPropertyNode {
    type Error = String;

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        match value.node {
            NodeKind::ObjectProperty(node) => Ok(node),
            _ => Err("".to_string()),
        }
    }
}

//impl Into<NodeKind> for &StringLiteralNode {
//    fn into(self) -> NodeKind {
//        NodeKind::StringLiteral(*self)
//    }
//}

#[derive(Clone, PartialEq)]
pub struct StringLiteralNode {
    pub value: String,
}

impl Debug for StringLiteralNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpressionNode {
    pub callee: Box<Node>,
    pub arguments: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpressionNode {
    pub properties: Vec<ObjectPropertyNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPropertyNode {
    pub computed: bool,
    pub key: Box<Node>,
    pub value: Box<Node>,
}

// impl Debug for ObjectPropertyNode {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}: {}", self.key, self.value)
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclarationNode {
    pub name: Box<Node>,
    pub parent: Option<Box<Node>>,
    pub methods: Vec<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpressionNode {
    pub computed: bool,
    pub object: Box<Node>,
    pub property: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpressionNode {
    pub test: Box<Node>,
    pub consequent: Box<Node>,
    pub alternative: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramNode {
    pub statements: Vec<Node>,
}

#[derive(Clone, PartialEq)]
pub struct Node {
    pub node: NodeKind,
    pub start: Span,
    pub end: Span,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.node)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatementNode {
    pub init: Option<Box<Node>>,
    pub test: Option<Box<Node>>,
    pub update: Option<Box<Node>>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpressionNode {
    pub callee: Box<Node>,
    pub params: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatementNode {
    pub expression: Box<Node>,
}

impl From<IdentifierNode> for NodeKind {
    fn from(value: IdentifierNode) -> Self {
        NodeKind::Identifier(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationNode {
    pub name: Box<IdentifierNode>,
    pub arguments: Vec<FunctionArgument>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpressionNode {
    pub arguments: Vec<FunctionArgument>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub name: String,
    pub default_value: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOperator {
    AddEqual,
    SubEqual,
    DivEqual,
    MulEqual,
    ExponentiationEqual,
    Equal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpressionNode {
    pub left: Box<Node>,
    pub operator: AssignmentOperator,
    pub right: Box<Node>,
}

impl TryFrom<&Token> for AssignmentOperator {
    type Error = String;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::PlusEqual => Ok(Self::AddEqual),
            Token::MinusEqual => Ok(Self::SubEqual),
            Token::MulEqual => Ok(Self::MulEqual),
            Token::MulMulEqual => Ok(Self::ExponentiationEqual),
            Token::DivEqual => Ok(Self::DivEqual),
            Token::Equal => Ok(Self::Equal),
            _ => Err("Cannot convert token kind to assignment operator".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatementNode {
    pub condition: Box<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatementNode {
    pub condition: Box<Node>,
    pub then_branch: Box<Node>,
    pub else_branch: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteralNode {
    value: f64,
}

#[derive(Clone, PartialEq)]
pub struct IdentifierNode {
    pub id: String,
}

impl Debug for IdentifierNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatementNode {
    pub statements: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Div,
    Mul,
    MulMul,
    LogicalOr,
    LogicalAnd,
    MoreThan,
    MoreThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equality,
    StrictEquality,
    Inequality,
    StrictInequality,
}

impl TryFrom<&Token> for BinaryOperator {
    type Error = String;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(Self::Add),
            Token::Minus => Ok(Self::Sub),
            Token::Mul => Ok(Self::Mul),
            Token::MulMul => Ok(Self::MulMul),
            Token::Div => Ok(Self::Div),
            Token::Or => Ok(Self::LogicalOr),
            Token::And => Ok(Self::LogicalAnd),
            Token::LessThan => Ok(Self::LessThan),
            Token::LessThanOrEqual => Ok(Self::LessThanOrEqual),
            Token::MoreThan => Ok(Self::MoreThan),
            Token::MoreThanOrEqual => Ok(Self::MoreThanOrEqual),
            Token::Equality => Ok(Self::Equality),
            Token::StrictEquality => Ok(Self::StrictEquality),
            Token::Inequality => Ok(Self::Inequality),
            Token::StrictInequality => Ok(Self::StrictInequality),
            _ => Err("Cannot convert token kind to binary operator".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpressionNode {
    pub left: Box<Node>,
    pub operator: BinaryOperator,
    pub right: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableDeclarationKind {
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationNode {
    pub kind: VariableDeclarationKind,
    pub id: String,
    pub value: Option<Box<Node>>,
}

trait FormatNode {
    fn format(&self, ident: u8, level: u8) -> String;
}

impl FormatNode for BinaryOperator {
    fn format(&self, _: u8, _: u8) -> String {
        match self {
            BinaryOperator::Add => "+".to_string(),
            BinaryOperator::Sub => "-".to_string(),
            BinaryOperator::Div => "/".to_string(),
            BinaryOperator::Mul => "*".to_string(),
            BinaryOperator::LogicalOr => "||".to_string(),
            BinaryOperator::LogicalAnd => "&&".to_string(),
            BinaryOperator::MoreThan => ">".to_string(),
            BinaryOperator::MoreThanOrEqual => ">=".to_string(),
            BinaryOperator::LessThan => "<".to_string(),
            BinaryOperator::LessThanOrEqual => "<=".to_string(),
            BinaryOperator::Equality => "==".to_string(),
            BinaryOperator::StrictEquality => "===".to_string(),
            BinaryOperator::Inequality => "!=".to_string(),
            BinaryOperator::StrictInequality => "!==".to_string(),
            BinaryOperator::MulMul => "**".to_string(),
        }
    }
}

impl FormatNode for VariableDeclarationKind {
    fn format(&self, _: u8, _: u8) -> String {
        match self {
            VariableDeclarationKind::Let => "let".to_string(),
            VariableDeclarationKind::Const => "const".to_string(),
        }
    }
}

impl FormatNode for AssignmentOperator {
    fn format(&self, _: u8, _: u8) -> String {
        match self {
            AssignmentOperator::AddEqual => "+=".to_string(),
            AssignmentOperator::SubEqual => "-=".to_string(),
            AssignmentOperator::DivEqual => "/=".to_string(),
            AssignmentOperator::MulEqual => "*=".to_string(),
            AssignmentOperator::Equal => "=".to_string(),
            AssignmentOperator::ExponentiationEqual => "**=".to_string(),
        }
    }
}

impl FormatNode for Box<Node> {
    fn format(&self, ident: u8, level: u8) -> String {
        self.as_ref().node.format(ident, level)
    }
}

impl FormatNode for Option<Box<Node>> {
    fn format(&self, ident: u8, level: u8) -> String {
        self.as_ref().map_or("".to_string(), |x| x.node.format(ident, level))
    }
}

impl FormatNode for NodeKind {
    fn format(&self, ident: u8, level: u8) -> String {
        let whitespaces = " ".repeat((ident * level) as usize);

        match self {
            NodeKind::StringLiteral(node) => format!("\'{}\'", node.value),
            NodeKind::NumberLiteral(value) => format!("{value}"),
            NodeKind::BooleanLiteral(value) => format!("{value}"),
            NodeKind::NullLiteral => "null".to_string(),
            NodeKind::UndefinedLiteral => "undefined".to_string(),
            NodeKind::Identifier(value) => format!("{}", value.id),
            NodeKind::BinaryExpression(node) => {
                let left = node.left.node.format(ident, level);
                let right = node.right.node.format(ident, level);
                let operator = node.operator.format(ident, level);

                return format!("{left} {operator} {right}");
            }
            NodeKind::VariableDeclaration(node) => {
                let variable_kind = node.kind.format(ident, level);
                let variable_name = &node.id;
                let variable_value = node.value.format(ident, level);
                return format!("{variable_kind} {variable_name} = {variable_value};");
            }
            NodeKind::AssignmentExpression(node) => {
                let assignment_kind = &node.operator.format(ident, level);
                let left = node.left.format(ident, level);
                let right = node.right.format(ident, level);
                return format!("{left} {assignment_kind} {right}");
            }
            NodeKind::BlockStatement(node) => {
                let formatted_statements: Vec<String> = node.statements
                    .iter()
                    .map(|x| {
                        let formatted_statement = x.node.format(ident, level);
                        format!("{}{}", whitespaces, formatted_statement.as_str())
                    })
                    .collect();
                let formatted_statements = formatted_statements.join("\n");
                return format!("{{\n{formatted_statements}\n{}}}", " ".repeat((ident * (level - 1)) as usize));
            }
            NodeKind::IfStatement(node) => {
                let condition = node.condition.as_ref().node.format(ident, level);
                let then_branch = node.then_branch.as_ref().node.format(ident, level);
                let else_branch = format!("else {}", node.else_branch.format(ident, level));

                return format!("if ({condition}) {then_branch} {else_branch}");
            }
            NodeKind::WhileStatement(node) => {
                let condition = node.condition.format(ident, level);
                let body = node.body.format(ident, level);
                return format!("while ({condition}) {body}");
            }
            NodeKind::ForStatement(node) => {
                let init = node.init.format(ident, level);
                let test = node.test.format(ident, level);
                let update = node.update.format(ident, level);
                let body = node.body.format(ident, level + 1);
                return format!("for ({init} {test}; {update}) {body}");
            }
            NodeKind::FunctionDeclaration(node) => {
                let function_name = &node.name.id;
                let function_args = &node.arguments.iter().enumerate().fold("".to_string(), |mut acc, (i, a)| {
                    let argument_name = &a.name;
                    let argument_default_value = a
                        .default_value
                        .as_ref()
                        .map_or("".to_string(), |x| format!(" = {}", x.node.format(ident, level)));
                    let comma = if i != node.arguments.len() - 1 { ", " } else { "" };
                    acc.push_str(format!("{argument_name}{argument_default_value}{}", comma).as_str());
                    return acc;
                });
                let function_body = &node.body.format(ident, level + 1);
                return format!("{whitespaces}function {function_name}({function_args}) {function_body}\n");
            }
            NodeKind::ReturnStatement(node) => {
                return format!("return {};", node.expression.node.format(ident, level));
            }
            NodeKind::CallExpression(node) => {
                format!("{}({})", node.callee.format(ident, level), node.params.iter().map(|x| x.node.format(ident, level)).collect::<Vec<String>>().join(", "))
            }
            NodeKind::ProgramStatement(node) => {
                let mut result = String::new();
                node.statements
                    .iter()
                    .for_each(|x| result.push_str(x.node.format(ident, level).as_str()));
                return result;
            }
            NodeKind::ConditionalExpression(node) => {
                let condition = node.test.format(ident, level);
                let consequent = node.consequent.node.format(ident, level);
                let alternative = node.alternative.node.format(ident, level);
                return format!("{condition} ? {consequent} : {alternative}");
            }
            NodeKind::MemberExpression(node) => {
                let obj_str = node.object.format(ident, level);
                let prop = node.property.format(ident, level);

                if node.computed {
                    format!("{}[{}]", obj_str, prop)
                } else {
                    format!("{}.{}", obj_str, prop)
                }
            },
            NodeKind::ClassDeclaration(node) => {
                let class_name = node.name.format(ident, level);
                let parent_class = match &node.parent {
                    Some(node) => {
                        let parent_class_name = node.format(ident, level);
                        format!(" extends {parent_class_name}")
                    }
                    None => "".to_string()
                };
                let class_body: Vec<String> = node.methods.iter().map(|x| {
                    format!("{}{}", whitespaces, x.node.format(ident, level + 1))
                }).collect();
                let class_body = class_body.join("\n");
                format!("class {class_name}{parent_class} {{\n{class_body}}}\n")
            },
            NodeKind::ObjectProperty(_) => todo!(),
            NodeKind::ObjectExpression(_) => todo!(),
            NodeKind::NewExpression(node) => {
                let callee = node.callee.format(ident, level);
                let arguments: Vec<String> = node.arguments.iter().map(|x| x.node.format(ident, level)).collect();
                let arguments = arguments.join(", ");
                format!("new {callee}({arguments})")
            }
            NodeKind::ThisExpression => "this".to_string(),
            NodeKind::FunctionExpression(_) => todo!(),
            NodeKind::ExpressionStatement(node) => format!("{};", node.format(ident, level))
        }
    }
}

pub fn format_ast(node: &NodeKind, ident: u8) -> String {
    return node.format(ident, 0);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser;

    const IDENT: u8 = 2;

    #[test]
    fn test_format_code() {
        let code = "function add(a=1,b=3){return a+b;}";

        let ast = parser::Parser::parse_code_to_ast(code)
          .expect("Error occured during parsing");

        assert_eq!(format_ast(&ast.node, IDENT), "function add(a = 1, b = 3) {return a + b;}");
    }

    #[test]
    fn few_levels() {
        let code = "class User {
  constructor(name, age) {
    this.name = name;
    this.age = age;
  }

  printUser() {
    console.log(this.name, this.age);
  }
}

let user = new User('Anton', 26)";

        let ast = parser::Parser::parse_code_to_ast(code)
            .expect("Error occured during parsing");

        println!("{}", format_ast(&ast.node, IDENT));
        assert_eq!(format_ast(&ast.node, IDENT), "function add(a = 1, b = 3) {return a + b;}");
    }
}
