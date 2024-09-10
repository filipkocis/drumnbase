use std::collections::HashMap;

use crate::{syntax::ast::{Node, Type}, basics::row::{Value, NumericValue}, database::database::{FunctionBody, Function}};

use super::Runner;

impl Runner {
    pub(super) fn eval_function(&self, name: &str, parameters: &Vec<(String, Type)>, return_type: &Type, block: &Box<Node>) -> Result<Value, String> {
        let function = Function {
            name: name.to_string(),
            args: parameters.clone(),
            body: FunctionBody::Custom(*block.clone())
        };

        let mut database = self.database.borrow_mut();
        if database.functions.contains_key(name) {
            return Err(format!("Functin '{}' already exists", name))
        }

        database.functions.insert(name.to_string(), function);

        Ok(Value::Null)
    }

    pub(super) fn eval_call(&self, name: &str, arguments: &Vec<Node>) -> Result<Value, String> {
        let arguments = arguments.iter().map(|arg| self.run(arg)).collect::<Result<Vec<Value>, String>>()?; 
        let database = self.database.borrow();
        
        if let Some(function) = database.functions.get(name) {
            if function.args.len() != arguments.len() {
                return Err(format!("Function '{}' expects {} arguments, got {}", name, function.args.len(), arguments.len()));
            }

            self.execute_function(function, arguments)
        } else {
            Err(format!("Function '{}' not found", name))
        }
    }

    fn execute_function(&self, function: &Function, arguments: Vec<Value>) -> Result<Value, String> {
        let body = match &function.body {
            FunctionBody::Custom(body) => body,
            FunctionBody::BuiltIn(function) => return function(self.database.clone(), &arguments),
        };

        let mut variables = self.variables.borrow_mut();
        let mut previous = HashMap::new();
        for ((param_name, param_type), value) in function.args.iter().zip(arguments) {
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
