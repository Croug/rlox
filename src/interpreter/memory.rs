use std::collections::HashMap;

use crate::{lexer::token::Token, parser::ast};

use super::RuntimeError;

pub struct Memory {
    map: HashMap<String, ast::LiteralValue>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, identifier: Token) -> Result<ast::LiteralValue, RuntimeError> {
        match self.map.get(&identifier.token_type.to_string()) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError::new(
                identifier.clone(),
                format!(
                    "Undefined variable '{token}'",
                    token = identifier.token_type
                ),
            )),
        }
    }

    pub fn set(&mut self, identifier: Token, value: ast::LiteralValue) {
        self.map.insert(identifier.token_type.to_string(), value);
    }
}
