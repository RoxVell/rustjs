use crate::scanner::{TokenKind, Scanner};
use crate::node_type::*;

pub struct Parser {
  current_token: Option<TokenKind>,
  scanner: Scanner,
}

impl Default for Parser {
    fn default() -> Self {
        Self { current_token: None, scanner: Scanner::new("".to_string())}
    }
}

impl Parser {
  pub fn parse(&mut self, source: &str) -> Result<Node, String> {
    self.scanner = Scanner::new(source.to_string());

    let mut statements: Vec<Node> = vec![];

    while let Some(token) = self.scanner.next_token() {
      self.current_token = Some(token);

      match self.current_token {
        Some(TokenKind::LetKeyword) | Some(TokenKind::ConstKeyword) => statements.push(self.parse_variable_declaration().unwrap()),
        _ => statements.push(self.parse_expression_statement().unwrap()),
      }
    }

    return Ok(Node::BlockStatement(BlockStatementNode {statements}));
  }

  fn parse_identifier(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::Identifier(id)) = &self.current_token {
      let id = id.clone();
      self.next_token();
      return Ok(Node::Identifier(IdentifierNode { id: id }));
    }

    unreachable!()
  }

  fn parse_variable_declaration(&mut self) -> Result<Node, String> {
    let variable_kind = match self.current_token {
      Some(TokenKind::LetKeyword) => VariableDeclarationKind::Let,
      Some(TokenKind::ConstKeyword) => VariableDeclarationKind::Const,
      _ => unreachable!()
    };

    self.next_token();

    if let Some(TokenKind::Identifier(id)) = &self.current_token {
      let id = id.clone();
      self.next_token();

      if let Some(TokenKind::Equal) = &self.current_token {
        self.next_token();
        let expression = self.parse_expression().expect("Expect variable initialization expression");
        return Ok(Node::VariableDeclaration(VariableDeclarationNode {  id: id.to_string(),  kind: variable_kind,  value: Box::new(expression)}));
      }
    } else {
      return Err("Identifier is missing in variable declaration".to_string());
    }

    unreachable!()
  }

  fn parse_expression_statement(&mut self) -> Result<Node, String> {
    let expression = self.parse_expression();

    if let Some(TokenKind::Semicolon) = self.current_token {
      self.next_token();
    } else {
      return Err("Expected semicolon".to_string());
    }

    return expression;
  }

  fn parse_expression(&mut self) -> Result<Node, String> {
    return self.parse_addition_binary_expression();
  }

  fn parse_addition_binary_expression(&mut self) -> Result<Node, String> {
    return self.parse_binary_expression(&Self::parse_multiplicative_binary_expression, vec![&TokenKind::Plus, &TokenKind::Minus]);
  }

  fn parse_multiplicative_binary_expression(&mut self) -> Result<Node, String> {
    return self.parse_binary_expression(&Self::parse_primary_expression, vec![&TokenKind::Mul, &TokenKind::Div]);
  }

  fn parse_binary_expression(&mut self, side_expression_fn: &impl Fn(&mut Self) -> Result<Node, String>, tokens: Vec<&TokenKind>) -> Result<Node, String> {
    let mut left = side_expression_fn(self);

    while let Some(token) = &self.current_token {
      if !tokens.contains(&token) {
        break;
      }
      let operator = BinaryOperator::try_from(token).unwrap();
      self.next_token();
      let right = side_expression_fn(self);
      left = Ok(Node::BinaryExpression(BinaryExpressionNode {
        left: Box::new(left.unwrap()),
        operator: operator,
        right: Box::new(right.unwrap())
      }));
    }

    return left;
  }

  fn parse_primary_expression(&mut self) -> Result<Node, String> {
    match self.current_token {
      Some(TokenKind::Number(_)) => return self.parse_number_literal(),
      Some(TokenKind::String(_)) => return self.parse_string_literal(),
      Some(TokenKind::Boolean(_)) => return self.parse_bool_literal(),
      Some(TokenKind::Null) => return self.parse_null_literal(),
      Some(TokenKind::Undefined) => return self.parse_undefined_literal(),
      Some(TokenKind::OpenParen) => return self.parse_paranthesised_expression(),
      Some(TokenKind::Identifier(_)) => return self.parse_identifier(),
      _ => todo!()
    }
  }

  fn parse_paranthesised_expression(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::OpenParen) = self.current_token {
      self.next_token();
      let expression = self.parse_expression();
//      println!("{:#?}", self.current_token);
      if let Some(TokenKind::CloseParen) = self.current_token {
        self.next_token();
        return expression;
      } else {
        return Err("Close paren was not found".to_string());
      }
    }

    unreachable!()
  }

  fn parse_bool_literal(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::Boolean(value)) = &self.current_token {
      let value = value.clone();
      self.next_token();
      return Ok(Node::BooleanLiteral(if value == "true" { true } else { false }));
    }

    unreachable!()
  }

  fn parse_null_literal(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::Null) = &self.current_token {
      self.next_token();
      return Ok(Node::NullLiteral);
    }

    unreachable!()
  }

  fn parse_undefined_literal(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::Undefined) = &self.current_token {
      self.next_token();
      return Ok(Node::UndefinedLiteral);
    }

    unreachable!()
  }

  fn parse_string_literal(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::String(str)) = &self.current_token {
      let str = str.clone();
      self.next_token();
      return Ok(Node::StringLiteral(str));
    }

    unreachable!()
  }

  fn parse_number_literal(&mut self) -> Result<Node, String> {
    if let Some(TokenKind::Number(number)) = self.current_token {
      self.next_token();
      return Ok(Node::NumberLiteral(number));
    }

    unreachable!()
  }

  fn next_token(&mut self) {
    self.current_token = self.scanner.next_token();
  }

//  pub fn eat<T: TokenKind>(&'a mut self, token_kind: &TokenKind) -> Result<TokenKind, String> {
//    if std::mem::discriminant(&self.current_token.unwrap()) == std::mem::discriminant(token_kind) {
//      let temp = self.current_token.unwrap();
//      self.set_next_token();
//      return Ok(temp);
//    }
//
//    return Err(format!("Expected token {}, but got: {}", self.current_token.unwrap(), token_kind));
//  }
}