use std::{sync::{Arc, RwLock}};

use crate::{database::Database, basics::Value, syntax::ast::{Node, Type}, lock::UnsafeRwLock};

mod builtins;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: FunctionBody,
}

#[derive(Debug)]
pub enum FunctionBody {
    BuiltIn(BuiltIn),
    Custom(Node),
}

type BuiltIn = fn(UnsafeRwLock<Database>, &[Value]) -> Result<Option<Value>, String>;

impl Function {
    pub fn new(name: String, params: Vec<(String, Type)>, return_type: Type, body: FunctionBody) -> Self {
        Self {
            name,
            params,
            return_type,
            body,
        }
    }

    pub fn built_in(name: &str, params: Vec<(&str, Type)>, return_type: Type, body: BuiltIn) -> Self {
        Self::new(
            name.to_string(),
            params.into_iter().map(|(name, t)| (name.to_string(), t)).collect(),
            return_type,
            FunctionBody::BuiltIn(body)
        )
    }

    pub fn custom(name: &str, params: &[(String, Type)], return_type: &Type, body: &Node) -> Self {
        Self::new(
            name.to_string(),
            params.to_vec(),
            return_type.clone(),
            FunctionBody::Custom(body.clone())
        )
    }

    pub fn call(&self, db: UnsafeRwLock<Database>, args: &[Value]) -> Result<Option<Value>, String> {
        match &self.body {
            FunctionBody::BuiltIn(f) => f(db, args),
            // TODO: Implement custom function call
            FunctionBody::Custom(_) => unimplemented!("Cannot call custom function directly")
        }
    }
}
