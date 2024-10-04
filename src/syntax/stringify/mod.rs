use std::borrow::Cow;

use crate::{basics::{Value, Column, column::{ColumnType, NumericType, TextType, TimestampType}, value::{NumericValue}}, auth::RlsAction};

use super::ast::{Node, SDL, CreateSDL, Literal, Number, Statement, Type, Expression, Operator, Query, SelectQuery, InsertQuery, DeleteQuery, UpdateQuery, GrantSDL};

/// Indentation helper
fn spaces(indent: usize) -> String {
    " ".repeat(indent * 4)
}

/// Wrap the node in a block if it isn't already
fn blockify(node: &Node) -> Cow<Node> {
    if let Node::Block(_) = node {
        return Cow::Borrowed(node)
    }

    Cow::Owned(Node::Block(vec![node.clone()]))
}

/// escape an identifier if it contains whitespace -> "ident\n ident", ident
fn escapify(s: &str) -> String {
    if s.chars().any(|c| c.is_whitespace()) {
        format!("{:?}", s)
    } else {
        s.to_string()
    }
}

pub trait SchemaStringDeblock {
    fn deblock(&self) -> String;
}

impl SchemaStringDeblock for String {
    fn deblock(&self) -> String {
        if self.is_empty() {
            return self.to_string()
        }

        if self.chars().next().unwrap() != '{' || self.chars().last().unwrap() != '}' {
            return self.to_string()
        }

        let lines = self.lines().collect::<Vec<_>>();

        if lines.len() == 1 {
            if self.chars().nth(1).unwrap() == '}' {
                return "".to_string()
            }

            let s = &self[1..self.len() - 2].trim();
            return s.to_string()
        }

        let spaces = spaces(1);

        if self.chars().nth(1).unwrap() != '\n' ||
            lines.last().unwrap().trim() != "}" ||
            lines.len() == 2 {
            return self.to_string()
        }

        let all_indented = lines.iter().enumerate().all(|(i, line)| {
            if i == 0 || i == lines.len() - 1 { 
                return true 
            }

            line.starts_with(&spaces)
        });

        if !all_indented {
            return self.to_string()
        }

        let spaces_len= spaces.len();
        let mut s = "".to_string();
        for (i, line) in lines.iter().enumerate() {
            if i == 0 || i == lines.len() - 1 { 
                continue
            }

            s.push_str(&line[spaces_len..]);
            s.push_str("\n");
        }
        s.pop();

        s
    }
}

pub trait ToSchemaString {
    fn to_schema_string(&self, indent: usize) -> Result<String, String>;
}

impl ToSchemaString for Node {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        match self {
            Node::SDL(sdl) => sdl.to_schema_string(indent),
            Node::Block(nodes) => {
                let spaces = spaces(indent);
                let mut schema = format!("{}{{", spaces);

                // { }
                if nodes.len() == 0 {
                    schema.push_str(" }");
                    return Ok(schema)
                } 

                // { node }
                if nodes.len() == 1 {
                    schema.push(' ');
                    schema.push_str(&nodes[0].to_schema_string(0)?);
                    schema.push_str(" }");
                    return Ok(schema)
                }

                // {
                //   node; 
                //   node;
                // }
                schema.push_str("\n");
                for node in nodes {
                    schema.push_str(&node.to_schema_string(indent + 1)?);
                    schema.push_str(";\n");
                }

                schema.push_str(&format!("{}}}", spaces));
                Ok(schema)
            },
            Node::Value(value) => value.to_schema_string(indent),
            Node::Literal(literal) => literal.to_schema_string(indent),
            Node::Statement(statement) => statement.to_schema_string(indent),
            Node::Expression(expression) => expression.to_schema_string(indent),
            Node::Query(query) => query.to_schema_string(indent),
        }
    }
}

impl ToSchemaString for SDL {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        match self {
            SDL::Create(create) => create.to_schema_string(indent),
            // SDL::Drop(drop) => drop.to_schema_string(indent),
            SDL::Drop(_) => Err("Drop not implemented for schema string".to_string()),
            SDL::Grant(grant) => grant.to_schema_string(indent),
        } 
    }
}

impl ToSchemaString for CreateSDL {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let spaces = spaces(indent);

        let s = match self {
            CreateSDL::Database { name } => format!("{}create database {}", spaces, name),
            CreateSDL::Table { name, columns } => {
                let mut schema = format!("{}create table {} {{\n", spaces, name);

                for column in columns {
                    schema.push_str(&column.to_schema_string(indent + 1)?);
                    schema.push_str(";\n");
                }

                schema.push_str(format!("{}}}", spaces).as_str());
                schema
            },
            CreateSDL::RlsPolicy { table, policy } => {
                let action = match policy.action {
                    RlsAction::Select => "select",
                    RlsAction::Update => "update",
                    RlsAction::Insert => "insert",
                    RlsAction::Delete => "delete",
                    RlsAction::All => "all",
                };

                let node = blockify(&policy.condition).to_schema_string(indent)?.trim().to_owned();
                format!("{}create policy {:?} for {}.{} {}", spaces, policy.name, table, action, node)
            }
            CreateSDL::Role { name } => format!("{}create role {}", spaces, name),
            CreateSDL::User { name, password, is_superuser } => {
                let superuser = if *is_superuser { " superuser" } else { "" };
                format!("{}create user {}:{}{}", spaces, name, password, superuser)
            }
        };

        Ok(s)
    }
}

impl ToSchemaString for Column {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let mut schema = format!("{}{}: {}", spaces(indent), self.name, self.data_type.to_schema_string(0)?);

        if self.not_null { schema.push_str(", required") }
        if self.unique { schema.push_str(", unique") }
        if self._default.is_some() { 
            let default = self._default.as_ref().unwrap();
            schema.push_str(&format!(", default({})", default.to_schema_string(indent)?.trim()));
        }

        Ok(schema)
    }
}

impl ToSchemaString for ColumnType {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            ColumnType::Numeric(numeric) => match numeric {
                NumericType::IntU8 => "u8".to_string(),
                NumericType::IntU16 => "u16".to_string(),
                NumericType::IntU32 => "u32".to_string(),
                NumericType::IntU64 => "u64".to_string(),
                NumericType::IntI8 => "i8".to_string(),
                NumericType::IntI16 => "i16".to_string(),
                NumericType::IntI32 => "i32".to_string(),
                NumericType::IntI64 => "i64".to_string(),
                NumericType::Float32 => "f32".to_string(),
                NumericType::Float64 => "f64".to_string(),
            },
            ColumnType::Text(text) => match text {
                TextType::Char => "char".to_string(),
                TextType::Variable => "variable".to_string(),
                TextType::Fixed(length) => format!("fixed({})", length),
            },
            ColumnType::Timestamp(time) => match time {
                TimestampType::Seconds => "time(s)".to_string(),
                TimestampType::Milliseconds => "time(ms)".to_string(),
                TimestampType::Microseconds => "time(us)".to_string(),
                TimestampType::Nanoseconds => "time(ns)".to_string(),
            },
            ColumnType::Boolean => "bool".to_string(),
            _ => Err("Unsupported column type for schema string".to_string())?,
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Value {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            Value::Numeric(numeric) => match numeric {
                NumericValue::IntU8(n) => n.to_string(),
                NumericValue::IntU16(n) => n.to_string(),
                NumericValue::IntU32(n) => n.to_string(),
                NumericValue::IntU64(n) => n.to_string(),
                NumericValue::IntI8(n) => n.to_string(),
                NumericValue::IntI16(n) => n.to_string(),
                NumericValue::IntI32(n) => n.to_string(),
                NumericValue::IntI64(n) => n.to_string(),
                NumericValue::Float32(n) => n.to_string(),
                NumericValue::Float64(n) => n.to_string(),
            },
            Value::Text(text) => format!("{:?}", text),
            Value::Timestamp(time) => match time {
                _ => Err("Timestamp not implemented for schema string".to_string())?,
            },
            Value::Boolean(b) => b.to_string(),
            Value::Array(a) => {
                let mut s = "[".to_string();

                for v in a {
                    s.push_str(&v.to_schema_string(indent + 1)?.trim());
                    s.push_str(", ");
                }

                s.push_str("]");
                s
            },
            Value::Null => "null".to_string(),

            _ => Err("Unsupported value for schema string".to_string())?,
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Literal {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            Literal::Identifier(id) => id.to_string(),
            Literal::Number(number) => match number {
                Number::Int(n) => n.to_string(),
                Number::UInt(n) => n.to_string(),
                Number::Float(n) => n.to_string(),
            },
            Literal::String(s) => format!("{:?}", s),
            Literal::Boolean(b) => b.to_string(),
            Literal::Array(a) => {
                let mut s = "[".to_string();

                for v in a {
                    s.push_str(&v.to_schema_string(indent + 1)?.trim());
                    s.push_str(", ");
                }

                s.push_str("]");
                s
            }
            Literal::Null => "null".to_string(),
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Statement {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            Statement::Assignment { name, value } => format!("{} = {}", name, value.to_schema_string(indent)?.trim()),
            Statement::Expression(expression) => return expression.to_schema_string(indent),
            Statement::Function { name, parameters, return_type, block } => {
                let mut schema = format!("fn {}(", name);

                for (i, (name, data_type)) in parameters.iter().enumerate() {
                    schema.push_str(&format!("{}: {}", name, data_type.to_schema_string(0)?));
                    
                    if i < parameters.len() - 1 {
                        schema.push_str(", ");
                    }
                }

                schema.push_str(") ");
                if return_type != &Type::Void {
                    schema.push_str(&format!("-> {} ", return_type.to_schema_string(0)?));
                }
                schema.push_str(&block.to_schema_string(indent)?.trim());
                schema
            },
            Statement::Let { name, value } => format!("let {} = {}", name, value.to_schema_string(indent)?.trim()),
            Statement::Return(value) => format!("return {}", value.to_schema_string(indent)?.trim()),
            Statement::If { condition, then_block, else_block } => {
                let mut schema = format!("if {} ", condition.to_schema_string(indent)?.trim());

                schema.push_str(&then_block.to_schema_string(indent)?.trim());

                if let Some(else_block) = else_block {
                    schema.push_str(&format!(" else {}", else_block.to_schema_string(indent)?.trim()));
                }

                schema
            },
            Statement::While { condition, block } => 
                format!("while {} {}", condition.to_schema_string(indent)?.trim(), block.to_schema_string(indent)?.trim()),
            Statement::For { initializer, condition, action, block } =>
                format!("for ({}; {}; {}) {}", 
                    initializer.to_schema_string(indent)?.trim(),
                    condition.to_schema_string(indent)?.trim(),
                    action.to_schema_string(indent)?.trim(),
                    block.to_schema_string(indent)?.trim(),
                ),
            Statement::Loop { block } => format!("loop {}", block.to_schema_string(indent)?.trim()),
            Statement::Break => "break".to_string(),
            Statement::Continue => "continue".to_string(),
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Type {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            Type::Int => "int".to_string(),
            Type::UInt => "uint".to_string(),
            Type::Float => "float".to_string(),
            Type::String => "string".to_string(),
            Type::Boolean => "bool".to_string(),
            Type::Pointer(t) => format!("*{}", t.to_schema_string(0)?),
            Type::Array(t) => format!("[]{}", t.to_schema_string(0)?),
            _ => Err("Unsupported type for schema string".to_string())?,
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Expression {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            Expression::Binary { left, operator, right } => 
                format!("{} {} {}",
                    left.to_schema_string(indent)?.trim(),
                    operator.to_schema_string(0)?,
                    right.to_schema_string(indent)?.trim(),
                ),
            Expression::Unary { operator, right } => 
                format!("{}{}", right.to_schema_string(indent)?.trim(), operator.to_schema_string(0)?),
            Expression::Call { name, arguments } => {
                let mut schema = format!("{}(", name);

                for (i, arg) in arguments.iter().enumerate() {
                    schema.push_str(&arg.to_schema_string(indent)?.trim());

                    if i < arguments.len() - 1 {
                        schema.push_str(", ")
                    }
                }

                schema.push(')');
                schema
            },
            Expression::Literal(literal) => literal.to_schema_string(indent)?,

            _ => Err("Unsupported expression for schema string".to_string())?,
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Operator {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
            Operator::Mod => "%".to_string(),

            Operator::Eq => "==".to_string(),
            Operator::Ne => "!=".to_string(),
            Operator::Lt => "<".to_string(),
            Operator::Le => "<=".to_string(),
            Operator::Gt => ">".to_string(),
            Operator::Ge => ">=".to_string(),

            Operator::And => "&&".to_string(),
            Operator::Or => "||".to_string(),
            Operator::Not => "!".to_string(),

            Operator::BitAnd => "&".to_string(),
            Operator::BitOr => "|".to_string(),
            Operator::BitXor => "^".to_string(),
            Operator::BitNot => "~".to_string(),
            
            Operator::ShiftLeft => "<<".to_string(),
            Operator::ShiftRight => ">>".to_string(),

            Operator::Assign => "=".to_string(),
            Operator::AddAssign => "+=".to_string(),
            Operator::SubAssign => "-=".to_string(),
            Operator::MulAssign => "*=".to_string(),
            Operator::DivAssign => "/=".to_string(),
            Operator::ModAssign => "%=".to_string(),

            Operator::Inc => "++".to_string(),
            Operator::Dec => "--".to_string(),

            _ => Err("Unsupported operator for schema string".to_string())?,
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}

impl ToSchemaString for Query {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
         match self {
             Query::Select(select) => select.to_schema_string(indent),
             Query::Insert(insert) => insert.to_schema_string(indent),
             Query::Update(update) => update.to_schema_string(indent),
             Query::Delete(delete) => delete.to_schema_string(indent),
         }
    }
}

impl ToSchemaString for SelectQuery {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let mut schema = format!("{}query ", spaces(indent));
        schema.push_str(&self.table);
        schema.push_str(" select");

        for (i, column) in self.columns.iter().enumerate() {
            schema.push(' ');
            schema.push_str(column.to_schema_string(indent)?.trim());

            if i < self.columns.len() - 1 {
                schema.push(',');
            }
        }

        if let Some(where_clause) = &self.where_clause {
            schema.push_str(&format!(" where {}", where_clause.to_schema_string(indent)?.trim()));
        }

        if let Some(order) = &self.order {
            match order.as_ref() {
                Node::Expression(expr) => match expr {
                    Expression::Unary { operator, right } => {
                        let operator = match operator {
                            Operator::Inc => "asc",
                            Operator::Dec => "desc",
                            _ => Err("Invalid order operator".to_string())?,
                        };
                        schema.push_str(&format!(" order {} {}", right.to_schema_string(indent)?.trim(), operator));
                    },
                    _ => Err("Invalid order expression".to_string())?,
                }
                _ => Err("Invalid order node".to_string())?,
            }
        }

        if let Some(limit) = &self.limit {
            schema.push_str(&format!(" limit {}", limit))
        }

        if let Some(offset) = &self.offset {
            schema.push_str(&format!(" offset {}", offset))
        }

        if let Some(exclude) = &self.exclude {
            schema.push_str(&format!(" exclude {}", exclude.join(", ")))
        }

        Ok(schema)
    }
}

impl ToSchemaString for InsertQuery {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let mut schema = format!("{}query ", spaces(indent)); 
        schema.push_str(&self.table);
        schema.push_str(" insert");

        for (key, val) in &self.key_values {
            schema.push_str(&format!(" {}:{}", escapify(key), val.to_schema_string(indent)?.trim()));
        }

        Ok(schema)
    }
}

impl ToSchemaString for UpdateQuery {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let mut schema = format!("{}query ", spaces(indent)); 
        schema.push_str(&self.table);
        schema.push_str(" update");

        for (key, val) in &self.key_values {
            schema.push_str(&format!(" {}:{}", escapify(key), val.to_schema_string(indent)?.trim()));
        }

        if let Some(where_clause) = &self.where_clause {
            schema.push_str(&format!(" where {}", where_clause.to_schema_string(indent)?.trim()));
        }

        Ok(schema)
    }
}

impl ToSchemaString for DeleteQuery {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let mut schema = format!("{}query ", spaces(indent)); 
        schema.push_str(&self.table);
        schema.push_str(" delete");

        if let Some(where_clause) = &self.where_clause {
            schema.push_str(&format!(" where {}", where_clause.to_schema_string(indent)?.trim()));
        }

        Ok(schema)
    }
}

impl ToSchemaString for GrantSDL {
    fn to_schema_string(&self, indent: usize) -> Result<String, String> {
        let s = match self {
            GrantSDL::Action { object, object_name, actions, table, to } => {
                let actions = actions.join(", ");

                let mut schema = format!("grant {} {} {}", actions, object, object_name);

                if let Some(table) = table {
                    schema.push_str(&format!(" for {}", table));
                }

                schema.push_str(&format!(" for {}", to));
                schema
            },
            GrantSDL::Role { name, to } => format!("grant role {} for {}", name, to)
        };

        Ok(format!("{}{}", spaces(indent), s))
    }
}
