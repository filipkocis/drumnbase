use std::collections::HashMap;

use crate::{syntax::ast::{Node, Type}, basics::row::Value, function::{Function, FunctionBody}};

use super::Runner;

impl Runner {
    pub(super) fn eval_function(&self, name: &str, parameters: &Vec<(String, Type)>, return_type: &Type, block: &Box<Node>) -> Result<Option<Value>, String> {
        let function = Function::custom(name, parameters, return_type, block);

        let mut database = self.database.write().unwrap();
        if database.functions.contains_key(name) {
            return Err(format!("Function '{}' already exists", name))
        }

        database.functions.insert(name.to_string(), function);

        Ok(None)
    }

    pub(super) fn eval_call(&self, name: &str, arguments: &Vec<Node>) -> Result<Option<Value>, String> {
        let arguments = arguments.iter()
            .map(|arg| match self.run(arg) {
                    Ok(value) => match value {
                        Some(value) => Ok(value),
                        None => Err("Cannot pass a statement without a return value as an argument".to_string())
                    }
                    Err(e) => Err(e)
                }
            )
            .collect::<Result<Vec<Value>, String>>()?; 
        let database = self.database.read().unwrap();
        
        if let Some(function) = database.functions.get(name) {
            if function.params.len() != arguments.len() {
                return Err(format!("Function '{}' expects {} arguments, got {}", name, function.params.len(), arguments.len()));
            }

            self.execute_function(function, arguments)
        } else {
            Err(format!("Function '{}' not found", name))
        }
    }

    fn execute_function(&self, function: &Function, arguments: Vec<Value>) -> Result<Option<Value>, String> {
        let body = match &function.body {
            FunctionBody::Custom(body) => body,
            FunctionBody::BuiltIn(function) => return function(self.database.clone(), &arguments),
        };

        let mut variables = self.variables.borrow_mut();
        let mut previous = HashMap::new();
        for ((param_name, param_type), value) in function.params.iter().zip(arguments) {
            if !self.check_type(param_type, &value) {
                return Err(format!("Function '{}' expects argument '{}' to be of type '{:?}' but got '{:?}'",
                    function.name, param_name, param_type, self.get_type(&value)
                ));
            }

            let key = variables.insert(param_name.to_string(), value);
            previous.insert(param_name.to_string(), key);
        }

        drop(variables); // drop mutable borrow before recursive call
        let result = self.run(&body);
        
        let mut variables = self.variables.borrow_mut();
        for (param_name, key) in previous {
            if let Some(value) = key {
                variables.insert(param_name, value);
            } else {
                variables.remove(&param_name);
            }
        }

        result
    }
}
