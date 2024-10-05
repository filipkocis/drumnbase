use crate::{syntax::{ast::{Node, Type}, context::{RunnerContextScope, RunnerContextVariable}}, basics::Value, function::{Function, FunctionBody}};

use super::{Runner, Ctx, RunnerResult};

impl Runner {
    pub(super) fn eval_function(&self, name: &str, parameters: &Vec<(String, Type)>, return_type: &Type, block: &Box<Node>) -> RunnerResult {
        let function = Function::custom(name, parameters, return_type, block);

        let mut database = self.database.write().map_err(|_| "Cannot create functions when in read mode")?;
        if database.functions.contains_key(name) {
            return Err(format!("Function '{}' already exists", name))
        }

        database.functions.insert(name.to_string(), function);

        Ok(None)
    }

    pub(super) fn eval_call(&self, name: &str, arguments: &Vec<Node>, ctx: &Ctx) -> RunnerResult {
        let arguments = arguments.iter()
            .map(|arg| match self.run(arg, ctx) {
                    Ok(value) => match value {
                        Some(value) => Ok(value),
                        None => Err("Cannot pass a statement without a return value as an argument".to_string())
                    }
                    Err(e) => Err(e)
                }
            )
            .collect::<Result<Vec<Value>, String>>()?; 
        let database = self.database.read();
        
        if let Some(function) = database.functions.get(name) {
            if function.params.len() != arguments.len() {
                return Err(format!("Function '{}' expects {} arguments, got {}", name, function.params.len(), arguments.len()));
            }

            self.execute_function(function, arguments, ctx)
        } else {
            Err(format!("Function '{}' not found", name))
        }
    }

    fn execute_function(&self, function: &Function, arguments: Vec<Value>, ctx: &Ctx) -> RunnerResult {
        // TODO: add type checking for builtins
        let body = match &function.body {
            FunctionBody::Custom(body) => body,
            FunctionBody::BuiltIn(function) => return function(self.database.clone(), &arguments, ctx, self),
        };

        // TODO: create a new context so the fn doesn't have access to the outer scope (?)
        let ctx = &Ctx::scoped(ctx.clone());
        for ((param_name, param_type), value) in function.params.iter().zip(arguments) {
            if !self.check_type(param_type, &value) {
                return Err(format!("Function '{}' expects argument '{}' to be of type '{:?}' but got '{:?}'",
                    function.name, param_name, param_type, self.get_type(&value)
                ));
            }

            ctx.declare(param_name, value);
        }

        self.run(&body, ctx)
    }
}
