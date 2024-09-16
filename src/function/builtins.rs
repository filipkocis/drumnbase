use std::{cell::RefCell, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use crate::{database::database::Database, syntax::ast::Type, basics::row::{Value, TimestampValue, NumericValue}};

use super::Function;

impl Database {
    pub fn add_builtin_functions(&mut self) {
        let functions = vec![
            print(),
            println(),
            now(),
            floor(),
            ceil(),
            round(),
            abs(),
            sqrt(),
            pow(),
            len(),
        ];

        for function in functions {
            self.functions.insert(function.name.clone(), function);
        }
    }
}

fn print() -> Function {
    let name = "print";
    let params = vec![("values", Type::Any)];
    let return_type = Type::Void;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let values = args.get(0).ok_or("Expected argument 'values'")?;
        print!("{}", values);
        Ok(None)
    };

    Function::built_in(name, params, return_type, body)
}

fn println() -> Function {
    let name = "println";
    let params = vec![("values", Type::Any)];
    let return_type = Type::Void;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let values = args.get(0).ok_or("Expected argument 'values'")?;
        println!("{}", values);
        Ok(None)
    };

    Function::built_in(name, params, return_type, body)
}

fn now() -> Function {
    let name = "now";
    let params = vec![];
    let return_type = Type::UInt;

    let body = |_: Rc<RefCell<Database>>, _: &[Value]| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        Ok(Some(Value::Timestamp(TimestampValue::Milliseconds(now as u64))))
    };

    Function::built_in(name, params, return_type, body)
}

fn floor() -> Function {
    let name = "floor";
    let params = vec![("value", Type::Float)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Numeric(NumericValue::Float32(f))
                => Ok(Some(Value::Numeric(NumericValue::Float32(f.floor())))),
            Value::Numeric(NumericValue::Float64(f)) 
                => Ok(Some(Value::Numeric(NumericValue::Float64(f.floor())))),
            _ => Err("Expected argument 'value' to be of type 'float'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn ceil() -> Function {
    let name = "ceil";
    let params = vec![("value", Type::Float)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Numeric(NumericValue::Float32(f))
                => Ok(Some(Value::Numeric(NumericValue::Float32(f.ceil())))),
            Value::Numeric(NumericValue::Float64(f)) 
                => Ok(Some(Value::Numeric(NumericValue::Float64(f.ceil())))),
            _ => Err("Expected argument 'value' to be of type 'float'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn round() -> Function {
    let name = "round";
    let params = vec![("value", Type::Float), ("precision", Type::Int)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        let precision = args.get(1).ok_or("Expected argument 'precision'")?;
        match (value, precision) {
            (Value::Numeric(NumericValue::Float32(f)), Value::Numeric(NumericValue::IntI64(p))) => match p {
                0 => Ok(Some(Value::Numeric(NumericValue::Float32(f.round())))),
                _ => Ok(Some(Value::Numeric(NumericValue::Float32((f * 10.0_f32.powi(*p as i32)).round() / 10.0_f32.powi(*p as i32))))), 
            }
            (Value::Numeric(NumericValue::Float64(f)), Value::Numeric(NumericValue::IntI64(p))) => match p {
                0 => Ok(Some(Value::Numeric(NumericValue::Float64(f.round())))),
                _ => Ok(Some(Value::Numeric(NumericValue::Float64((f * 10.0_f64.powi(*p as i32)).round() / 10.0_f64.powi(*p as i32))))),
            }
            _ => Err("Expected arguments 'value' and 'precision' to be of type 'float'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn abs() -> Function {
    let name = "abs";
    let params = vec![("value", Type::Float)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Numeric(NumericValue::Float32(f))
                => Ok(Some(Value::Numeric(NumericValue::Float32(f.abs())))),
            Value::Numeric(NumericValue::Float64(f)) 
                => Ok(Some(Value::Numeric(NumericValue::Float64(f.abs())))),
            Value::Numeric(NumericValue::IntI64(i))
                => Ok(Some(Value::Numeric(NumericValue::IntI64(i.abs())))),
            // TODO signed
            _ => Err("Expected argument 'value' to be of type 'float' or 'signed'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn sqrt() -> Function {
    let name = "sqrt";
    let params = vec![("value", Type::Float)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Numeric(NumericValue::Float32(f))
                => Ok(Some(Value::Numeric(NumericValue::Float32(f.sqrt())))),
            Value::Numeric(NumericValue::Float64(f)) 
                => Ok(Some(Value::Numeric(NumericValue::Float64(f.sqrt())))),
            _ => Err("Expected argument 'value' to be of type 'float'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn pow() -> Function {
    let name = "pow";
    let params = vec![("base", Type::Float), ("exponent", Type::Float)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let base = args.get(0).ok_or("Expected argument 'base'")?;
        let exponent = args.get(1).ok_or("Expected argument 'exponent'")?;
        match (base, exponent) {
            (Value::Numeric(NumericValue::Float32(b)), Value::Numeric(NumericValue::Float32(e)))
                => Ok(Some(Value::Numeric(NumericValue::Float32(b.powf(*e))))),
            (Value::Numeric(NumericValue::Float64(b)), Value::Numeric(NumericValue::Float64(e))) 
                => Ok(Some(Value::Numeric(NumericValue::Float64(b.powf(*e))))),
            _ => Err("Expected arguments 'base' and 'exponent' to be of type 'float'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

// fn min()
// fn max()
// fn random()

fn len() -> Function {
    let name = "len";
    let params = vec![("value", Type::Any)];
    let return_type = Type::UInt;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Text(s) => Ok(Some(Value::Numeric(NumericValue::IntU64(s.len() as u64)))),
            Value::Array(a) => Ok(Some(Value::Numeric(NumericValue::IntU64(a.len() as u64)))),
            _ => Err("Expected argument 'value' to be of type 'text' or 'array'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}
