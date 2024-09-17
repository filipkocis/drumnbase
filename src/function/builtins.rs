use std::{cell::RefCell, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use crate::{database::database::Database, syntax::ast::Type, basics::row::{Value, TimestampValue, NumericValue}, random::Random};

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
            random(),
            random_range(),
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
    let params = vec![("value", Type::Any)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Numeric(n)
                => Ok(Some(Value::Numeric(NumericValue::Float64(n.to_f64().abs())))),
            _ => Err("Expected argument 'value' to be of type 'number'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn sqrt() -> Function {
    let name = "sqrt";
    let params = vec![("value", Type::Any)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let value = args.get(0).ok_or("Expected argument 'value'")?;
        match value {
            Value::Numeric(n) 
                => Ok(Some(Value::Numeric(NumericValue::Float64(n.to_f64().abs())))),
            _ => Err("Expected argument 'value' to be of type 'number'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

fn pow() -> Function {
    let name = "pow";
    let params = vec![("base", Type::Any), ("exponent", Type::Any)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let base = args.get(0).ok_or("Expected argument 'base'")?;
        let exponent = args.get(1).ok_or("Expected argument 'exponent'")?;
        match (base, exponent) {
            (Value::Numeric(n1), Value::Numeric(n2))
                => Ok(Some(Value::Numeric(NumericValue::Float64(n1.to_f64().powf(n2.to_f64()))))),
            _ => Err("Expected arguments 'base' and 'exponent' to be of type 'number'".to_string())
        }
    };

    Function::built_in(name, params, return_type, body)
}

// fn min()
// fn max()

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

fn random() -> Function {
    let name = "random";
    let params = vec![];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, _: &[Value]| {
        let random = Random::gen();

        Ok(Some(Value::Numeric(NumericValue::Float64(random))))
    };

    Function::built_in(name, params, return_type, body)
}

fn random_range() -> Function {
    let name = "random_range";
    let params = vec![("min", Type::Float), ("max", Type::Float)];
    let return_type = Type::Float;

    let body = |_: Rc<RefCell<Database>>, args: &[Value]| {
        let min = args.get(0).ok_or("Expected argument 'min'")?;
        let max = args.get(1).ok_or("Expected argument 'max'")?;

        let min = match min {
            Value::Numeric(n) => n.to_f64(),
            _ => return Err("Expected argument 'min' to be of type 'number'".to_string())
        };
        let max = match max {
            Value::Numeric(n) => n.to_f64(),
            _ => return Err("Expected argument 'max' to be of type 'number'".to_string())
        };

        let random = Random::gen_range(min, max);

        Ok(Some(Value::Numeric(NumericValue::Float64(random))))
    };

    Function::built_in(name, params, return_type, body)
}
