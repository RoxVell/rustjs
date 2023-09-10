use crate::scanner::{Span, Token, TokenKind};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::process::id;

#[derive(Debug, Clone, PartialEq)]
pub enum AstStatement {
    ProgramStatement(ProgramNode),
    VariableDeclaration(VariableDeclarationNode),
    BlockStatement(BlockStatementNode),
    WhileStatement(WhileStatementNode),
    ForStatement(ForStatementNode),
    FunctionDeclaration(FunctionDeclarationNode),
    ReturnStatement(ReturnStatementNode),
    ExpressionStatement(AstExpression),
    IfStatement(IfStatementNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstExpression {
    StringLiteral(StringLiteralNode),
    NumberLiteral(NumberLiteralNode),
    BooleanLiteral(BooleanLiteralNode),
    NullLiteral(Token),
    UndefinedLiteral(Token),
    ThisExpression(Token),
    Identifier(IdentifierNode),
    BinaryExpression(BinaryExpressionNode),
    AssignmentExpression(AssignmentExpressionNode),
    FunctionExpression(FunctionExpressionNode),
    CallExpression(CallExpressionNode),
    ConditionalExpression(ConditionalExpressionNode),
    MemberExpression(MemberExpressionNode),
    NewExpression(NewExpressionNode),
    ObjectExpression(ObjectExpressionNode),
    ClassDeclaration(ClassDeclarationNode),
}

impl Into<AstStatement> for AstExpression {
    fn into(self) -> AstStatement {
        AstStatement::ExpressionStatement(self)
    }
}

// #[derive(Clone, PartialEq)]
// struct NumberLiteralExpression {
//     value: f64,
//     token: Token,
// }

// impl TryFrom<Node> for ObjectPropertyNode {
//     type Error = String;
//
//     fn try_from(value: Node) -> Result<Self, Self::Error> {
//         match value.node {
//             NodeKind::ObjectProperty(node) => Ok(node),
//             _ => Err("".to_string()),
//         }
//     }
// }

//impl Into<NodeKind> for &StringLiteralNode {
//    fn into(self) -> NodeKind {
//        NodeKind::StringLiteral(*self)
//    }
//}

#[derive(Clone, PartialEq)]
pub struct StringLiteralNode {
    pub value: String,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanLiteralNode {
    pub value: bool,
    pub token: Token,
}

impl Debug for StringLiteralNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpressionNode {
    pub callee: Box<AstExpression>,
    pub arguments: Vec<AstExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpressionNode {
    pub properties: Vec<ObjectPropertyNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPropertyNode {
    pub computed: bool,
    pub key: Box<AstExpression>,
    pub value: Box<AstExpression>,
}

impl FormatNode for ObjectPropertyNode {
    fn format(&self, ident: u8, level: u8) -> String {
        let mut key = self.key.format(ident, level);

        if self.computed {
            key = format!("[{key}]");
        }

        let value = self.value.format(ident, level);

        format!("{}{key}: {value}", " ".repeat((ident * level) as usize))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclarationNode {
    pub name: Box<IdentifierNode>,
    pub parent: Option<Box<IdentifierNode>>,
    pub methods: Vec<Box<ClassMethodNode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethodNode {
    pub function_signature: FunctionSignature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpressionNode {
    pub computed: bool,
    pub object: Box<AstExpression>,
    pub property: Box<AstExpression>, // TODO: type
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpressionNode {
    pub test: Box<AstExpression>,
    pub consequent: Box<AstExpression>,
    pub alternative: Box<AstExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramNode {
    pub statements: Vec<AstStatement>,
}

// #[derive(Clone, PartialEq)]
// pub struct Node {
//     pub node: NodeKind,
//     pub start: Span,
//     pub end: Span,
// }

// impl Debug for Node {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{:?}", self.node)
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatementNode {
    pub init: Option<Box<AstStatement>>,
    pub test: Option<Box<AstExpression>>,
    pub update: Option<Box<AstExpression>>,
    pub body: Box<AstStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpressionNode {
    pub callee: Box<AstExpression>,
    pub params: Vec<AstExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatementNode {
    pub expression: Box<AstExpression>,
}

impl From<IdentifierNode> for AstStatement {
    fn from(value: IdentifierNode) -> Self {
        AstExpression::Identifier(value).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationNode {
    pub function_signature: FunctionSignature
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub name: Box<IdentifierNode>,
    pub arguments: Vec<FunctionArgument>,
    pub body: Box<AstStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpressionNode {
    pub arguments: Vec<FunctionArgument>,
    pub body: Box<AstStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub name: IdentifierNode,
    pub default_value: Option<Box<AstExpression>>,
}

impl FormatNode for FunctionArgument {
    fn format(&self, ident: u8, level: u8) -> String {
        let default_value = match &self.default_value {
            Some(value) => format!(" = {}", value.format(ident, level)),
            None => "".to_string(),
        };
        format!("{}{}", self.name.id, default_value)
    }
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
    pub left: Box<AstExpression>,
    pub operator: AssignmentOperator,
    pub right: Box<AstExpression>,
}

impl TryFrom<&TokenKind> for AssignmentOperator {
    type Error = String;

    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::PlusEqual => Ok(Self::AddEqual),
            TokenKind::MinusEqual => Ok(Self::SubEqual),
            TokenKind::MulEqual => Ok(Self::MulEqual),
            TokenKind::MulMulEqual => Ok(Self::ExponentiationEqual),
            TokenKind::DivEqual => Ok(Self::DivEqual),
            TokenKind::Equal => Ok(Self::Equal),
            _ => Err("Cannot convert token kind to assignment operator".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatementNode {
    pub condition: Box<AstExpression>,
    pub body: Box<AstStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatementNode {
    pub condition: Box<AstExpression>,
    pub then_branch: Box<AstStatement>,
    pub else_branch: Option<Box<AstStatement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteralNode {
    pub value: f64,
    pub token: Token,
}

impl Into<AstExpression> for NumberLiteralNode {
    fn into(self) -> AstExpression {
        AstExpression::NumberLiteral(self)
    }
}

impl Into<AstStatement> for NumberLiteralNode {
    fn into(self) -> AstStatement {
        AstExpression::NumberLiteral(self).into()
    }
}

#[derive(Clone, PartialEq)]
pub struct IdentifierNode {
    pub id: String,
    pub token: Token,
}

impl Debug for IdentifierNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier(\"{}\")", self.id)
    }
}

impl Into<AstExpression> for IdentifierNode {
    fn into(self) -> AstExpression {
        AstExpression::Identifier(self)
    }
}

// impl Debug for IdentifierNode {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.id)
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatementNode {
    pub statements: Vec<AstStatement>,
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

impl TryFrom<&TokenKind> for BinaryOperator {
    type Error = String;

    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Sub),
            TokenKind::Mul => Ok(Self::Mul),
            TokenKind::MulMul => Ok(Self::MulMul),
            TokenKind::Div => Ok(Self::Div),
            TokenKind::Or => Ok(Self::LogicalOr),
            TokenKind::And => Ok(Self::LogicalAnd),
            TokenKind::LessThan => Ok(Self::LessThan),
            TokenKind::LessThanOrEqual => Ok(Self::LessThanOrEqual),
            TokenKind::MoreThan => Ok(Self::MoreThan),
            TokenKind::MoreThanOrEqual => Ok(Self::MoreThanOrEqual),
            TokenKind::Equality => Ok(Self::Equality),
            TokenKind::StrictEquality => Ok(Self::StrictEquality),
            TokenKind::Inequality => Ok(Self::Inequality),
            TokenKind::StrictInequality => Ok(Self::StrictInequality),
            _ => Err("Cannot convert token kind to binary operator".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpressionNode {
    pub left: Box<AstExpression>,
    pub operator: BinaryOperator,
    pub right: Box<AstExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableDeclarationKind {
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationNode {
    pub kind: VariableDeclarationKind,
    pub id: IdentifierNode,
    pub value: Option<Box<AstExpression>>,
}

trait FormatNode {
    fn format(&self, ident: u8, level: u8) -> String;
}

impl FormatNode for FunctionSignature {
    fn format(&self, ident: u8, level: u8) -> String {
        let function_name = &self.name.id;
        let arguments_list: Vec<String> = self
            .arguments
            .iter()
            .map(|x| {
                let argument_name = &x.name.id;
                let argument_default_value =
                    x.default_value.as_ref().map_or("".to_string(), |x| {
                        format!(" = {}", x.format(ident, level))
                    });
                format!("{argument_name}{argument_default_value}")
            })
            .collect();
        let arguments = arguments_list.join(", ");
        let function_body = &self.body.format(ident, level + 1);
        return format!("{function_name}({arguments}) {function_body}\n");
    }
}

impl FormatNode for ClassMethodNode {
    fn format(&self, ident: u8, level: u8) -> String {
        let whitespaces = " ".repeat((ident * level) as usize);
        let function_signature_formatted = self.function_signature.format(ident, level);
        format!("{whitespaces}{function_signature_formatted}")
    }
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

impl<T: FormatNode> FormatNode for Box<T> {
    fn format(&self, ident: u8, level: u8) -> String {
        self.as_ref().format(ident, level)
    }
}

// impl FormatNode for Option<Box<AstStatement>> {
//     fn format(&self, ident: u8, level: u8) -> String {
//         self.as_ref()
//             .map_or("".to_string(), |x| x.node.format(ident, level))
//     }
// }

impl FormatNode for IdentifierNode {
    fn format(&self, ident: u8, level: u8) -> String {
        self.id.clone()
    }
}

impl FormatNode for AstExpression {
    fn format(&self, ident: u8, level: u8) -> String {
        let whitespaces = " ".repeat((ident * level) as usize);

        match self {
            AstExpression::StringLiteral(node) => format!("\'{}\'", node.value),
            AstExpression::NumberLiteral(node) => format!("{}", node.value),
            AstExpression::BooleanLiteral(node) => format!("{}", node.value),
            AstExpression::NullLiteral(_) => "null".to_string(),
            AstExpression::UndefinedLiteral(_) => "undefined".to_string(),
            AstExpression::Identifier(value) => format!("{}", value.id),
            AstExpression::BinaryExpression(node) => {
                let left = node.left.format(ident, level);
                let right = node.right.format(ident, level);
                let operator = node.operator.format(ident, level);

                return format!("{left} {operator} {right}");
            }
            AstExpression::AssignmentExpression(node) => {
                let assignment_kind = &node.operator.format(ident, level);
                let left = node.left.format(ident, level);
                let right = node.right.format(ident, level);
                return format!("{left} {assignment_kind} {right}");
            }
            AstExpression::ConditionalExpression(node) => {
                let condition = node.test.format(ident, level);
                let consequent = node.consequent.format(ident, level);
                let alternative = node.alternative.format(ident, level);
                return format!("{condition} ? {consequent} : {alternative}");
            }
            AstExpression::MemberExpression(node) => {
                let obj_str = node.object.format(ident, level);
                let prop = node.property.format(ident, level);

                if node.computed {
                    format!("{}[{}]", obj_str, prop)
                } else {
                    format!("{}.{}", obj_str, prop)
                }
            }
            AstExpression::ClassDeclaration(node) => {
                let class_name = node.name.format(ident, level);
                let parent_class = match &node.parent {
                    Some(node) => {
                        let parent_class_name = node.as_ref().format(ident, level);
                        format!(" extends {parent_class_name}")
                    }
                    None => "".to_string(),
                };
                let class_body: Vec<String> = node
                    .methods
                    .iter()
                    .map(|x| format!("{}{}", whitespaces, x.format(ident, level + 1)))
                    .collect();
                let class_body = class_body.join("\n");
                format!("class {class_name}{parent_class} {{\n{class_body}}}\n\n")
            }
            AstExpression::ObjectExpression(node) => {
                if node.properties.len() == 0 {
                    return format!("{{}}");
                }

                let properties_list: Vec<String> = node
                    .properties
                    .iter()
                    .map(|x| x.format(ident, level + 1))
                    .collect();
                let properties = properties_list.join(",\n");
                format!("{{\n{properties}\n{whitespaces}}}")
            }
            AstExpression::NewExpression(node) => {
                let callee = node.callee.format(ident, level);
                let arguments: Vec<String> = node
                    .arguments
                    .iter()
                    .map(|x| x.format(ident, level))
                    .collect();
                let arguments = arguments.join(", ");
                format!("new {callee}({arguments})")
            }
            AstExpression::ThisExpression(_) => "this".to_string(),
            AstExpression::FunctionExpression(node) => {
                let arguments_list: Vec<String> = node
                    .arguments
                    .iter()
                    .map(|x| x.format(ident, level))
                    .collect();
                let arguments = arguments_list.join("\n");
                let function_body = node.body.format(ident, level + 1);
                format!("function({arguments}) {function_body}")
            }
            AstExpression::CallExpression(node) => {
                format!(
                    "{}({})",
                    node.callee.format(ident, level),
                    node.params
                        .iter()
                        .map(|x| x.format(ident, level))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

impl<T: FormatNode> FormatNode for Option<T> {
    fn format(&self, ident: u8, level: u8) -> String {
        self.as_ref().map(|x| x.format(ident, level)).unwrap_or("".to_string())
    }
}

impl FormatNode for AstStatement {
    fn format(&self, ident: u8, level: u8) -> String {
        let whitespaces = " ".repeat((ident * level) as usize);

        match self {
            AstStatement::VariableDeclaration(node) => {
                let variable_kind = node.kind.format(ident, level);
                let variable_name = &node.id.format(ident, level);
                let variable_value = node.value.format(ident, level);
                return format!("{variable_kind} {variable_name} = {variable_value};");
            }

            AstStatement::BlockStatement(node) => {
                if node.statements.len() == 0 {
                    return format!("{{}}");
                }

                let formatted_statements: Vec<String> = node
                    .statements
                    .iter()
                    .map(|x| {
                        let formatted_statement = x.format(ident, level);
                        format!("{}{}", whitespaces, formatted_statement.as_str())
                    })
                    .collect();
                let formatted_statements = formatted_statements.join("\n");
                return format!(
                    "{{\n{formatted_statements}\n{}}}",
                    " ".repeat((ident * (level - 1)) as usize)
                );
            }
            AstStatement::IfStatement(node) => {
                let condition = node.condition.as_ref().format(ident, level);
                let then_branch = node.then_branch.as_ref().format(ident, level);
                let else_branch = format!("else {}", node.else_branch.format(ident, level));

                return format!("if ({condition}) {then_branch} {else_branch}");
            }
            AstStatement::WhileStatement(node) => {
                let condition = node.condition.format(ident, level);
                let body = node.body.format(ident, level);
                return format!("while ({condition}) {body}");
            }
            AstStatement::ForStatement(node) => {
                let init = node.init.format(ident, level);
                let test = node.test.format(ident, level);
                let update = node.update.format(ident, level);
                let body = node.body.format(ident, level + 1);
                return format!("for ({init} {test}; {update}) {body}");
            }
            AstStatement::FunctionDeclaration(node) => {
                let function_signature_formatted = node.function_signature.format(ident, level);
                return format!("{whitespaces}function {function_signature_formatted}\n");
            }
            AstStatement::ReturnStatement(node) => {
                return format!("return {};", node.expression.format(ident, level));
            }
            AstStatement::ProgramStatement(node) => {
                let mut result = String::new();
                node.statements
                    .iter()
                    .for_each(|x| result.push_str(x.format(ident, level).as_str()));
                return result;
            }
            AstStatement::ExpressionStatement(node) => format!("{};", node.format(ident, level)),
        }
    }
}

pub fn format_ast(node: &AstStatement, ident: u8) -> String {
    return node.format(ident, 0);
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::parser;
//
//     const IDENT: u8 = 2;
//
//     #[test]
//     fn test_format_code() {
//         let code =
//             "let a = {b: 5, [123]: 'Anton', [2 + 3]: true, qwe: {c: 10, b: {pp: function() {}}}}";
//
//         let ast = parser::Parser::parse_code_to_ast(code).expect("Error occured during parsing");
//
//         println!("{}", format_ast(&ast.node, IDENT));
//
//         assert_eq!(
//             format_ast(&ast.node, IDENT),
//             "function add(a = 1, b = 3) {return a + b;}"
//         );
//     }
//
//     #[test]
//     fn few_levels() {
//         let code = "class User {
//   constructor(name, age) {
//     this.name = name;
//     this.age = age;
//   }
//
//   printUser() {
//     console.log(this.name, this.age);
//   }
// }
//
// let user = new User('Anton', 26)";
//
//         let ast = parser::Parser::parse_code_to_ast(code).expect("Error occured during parsing");
//
//         println!("{}", format_ast(&ast.node, IDENT));
//         assert_eq!(
//             format_ast(&ast.node, IDENT),
//             "function add(a = 1, b = 3) {return a + b;}"
//         );
//     }
// }
