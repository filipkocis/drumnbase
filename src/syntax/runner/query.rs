use std::collections::HashMap;

use crate::{syntax::ast::{Query, InsertQuery, SelectQuery, UpdateQuery, DeleteQuery, Node, Literal, Operator, Expression}, basics::row::{Value, Row}};

use super::Runner;

impl Runner {
    pub(super) fn eval_query(&self, query: &Query) -> Result<Option<Value>, String> {
        let saved_scope = self.apply_query_scope(query);

        let result = match query {
            Query::Select(select) => self.eval_select(select),
            Query::Insert(insert) => self.eval_insert(insert),
            Query::Update(update) => self.eval_update(update),
            Query::Delete(delete) => self.eval_delete(delete),
            _ => Err(format!("Unsupported query type {:?}", query)),        
        };

        self.reset_scope(saved_scope);
        result
    }

    fn apply_query_scope(&self, query: &Query) -> HashMap<String, Option<Value>> {
        let mut saved_scope = HashMap::new(); 
        let database = self.database.borrow();
        let variables = self.variables.borrow();

        let table_name = Self::get_query_table(query);
        let table = database.get_table(table_name).unwrap();
        
        for column in &table.columns {
            saved_scope.insert(
                column.name.clone(), 
                variables.get(&column.name).cloned()
            );
        }

        saved_scope
    }

    fn get_query_table(query: &Query) -> &str {
        match query {
            Query::Select(q) => &q.table,
            Query::Insert(q) => &q.table,
            Query::Update(q) => &q.table,
            Query::Delete(q) => &q.table,
        }
    }

    fn eval_select(&self, select: &SelectQuery) -> Result<Option<Value>, String> {
        let database = self.database.borrow();
        let table = database.get_table(&select.table).unwrap();
        let column_map = table.get_column_map(&table.get_column_names()).unwrap();

        // (i, col) -> i is the index of the col in the result set, col is the value
        let mut special_columns = vec![];
        let mut column_names = select.columns.iter().enumerate().filter_map(|(i, node)| {
            match node {
                Node::Literal(literal) => match literal {
                    Literal::Identifier(n) if &*n == "*" => { special_columns.push((i, node)); None },
                    Literal::Identifier(name) => Some(name),
                    _ => { special_columns.push((i, node)); None }
                },
                _ => { special_columns.push((i, node)); None }
            }
        }).collect::<Vec<_>>();

        // check if columns exist in the table
        for name in &column_names {
            if table.get_column(name).is_none() {
                return Err(format!("Column '{}' does not exist in table '{}' in database '{}'", 
                        name, table.name, database.name))
            } 
        }

        // handle special 'select all' column
        let column_star_op = special_columns.iter()
            .position(|(_, node)| match node {
                Node::Literal(Literal::Identifier(name)) => name == "*",
                _ => false
            });
        if let Some(i) = column_star_op {
            let star_op = special_columns.remove(i);
            let star_op_i = star_op.0 - i;

            // get all columns which are not in column_names
            let columns = table.columns.iter().filter_map(|col| {
                if !column_names.iter().any(|name| *name == &col.name) { 
                    Some(&col.name) 
                }
                else { None }
            }).collect::<Vec<_>>();

            // insert columns in place of star_op from special_columns in column_names
            for (i, name) in columns.iter().enumerate() {
                column_names.insert(star_op_i + i, name);
            }

            // shift indexes in special_columns
            special_columns.iter_mut().for_each(|(i, _)| {
                if *i > star_op_i {
                    *i += columns.len() - 1;
                }
            });
        }

        // map column names to (i, name) where i is the index of the column in the table
        // filter out excluded columns
        let column_names = column_names.iter().enumerate().filter_map(|(i, name)| {
            if let Some(exclude) = &select.exclude {
                // skip column if it is in exclusion list
                if exclude.iter().any(|ex| ex == *name) { 
                    // shift indexes in special_columns
                    special_columns.iter_mut().for_each(|(j, _)| {
                        if *j > i { *j -= 1 }
                    });
                    return None
                }
            }
            
            table.columns.iter().position(|col| col.name == **name)
                .map(|i| (i, *name))
        }).collect::<Vec<_>>();

        // evaluate where clause on each row
        let mut row_indexes = vec![];
        for (i, row) in table.data.iter().enumerate() {
            if row.is_deleted() { continue }
            
            let mut variables = self.variables.borrow_mut(); 
            for (name, j) in &column_map {
                variables.insert(name.to_string(), row.get(*j).unwrap().clone());
            }
            drop(variables);

            let where_clause_result = match &select.where_clause {
                Some(node) => self.run(node),
                None => Ok(Some(Value::Boolean(true)))
            };

            match where_clause_result {
                Ok(Some(Value::Boolean(true))) => row_indexes.push(i),
                Ok(Some(Value::Boolean(false))) => (),
                Ok(_) => return Err("Where clause must return a boolean value".to_string()),
                Err(err) => return Err(err)
            };
        }

        // sort rows
        if let Some(order) = &select.order {
            let order = match order.as_ref() {
                Node::Expression(Expression::Unary { operator, right }) => {
                    let name = match **right {
                        Node::Literal(Literal::Identifier(ref name)) => name,
                        _ => return Err("Order node must be a column name".to_string())
                    };

                    let ascending = match operator {
                        Operator::Inc => true,
                        Operator::Dec => false,
                        _ => return Err("Order operator must be 'inc' or 'dec'".to_string())
                    };

                    // TODO: use check_exists in all uses of get_column
                    if table.get_column(name).is_none() {
                        return Err(format!("Column '{}' does not exist in table '{}'", name, table.name))
                    }

                    let i = table.get_column_index(name).unwrap();

                    (i, ascending)
                },
                _ => return Err("Order must be a unary expression".to_string())
            };

            // sort row indexes by order column
            row_indexes.sort_by(|&i, &j| {
                let a = table.data.get(i).expect("Cannot get row with row_index")
                    .get(order.0).expect("Cannot get row value with order index");
                let b = table.data.get(j).expect("Cannot get row with row_index")
                    .get(order.0).expect("Cannot get row value with order index");
                
                if order.1 { 
                    a.partial_cmp(b).expect("Cannot compare values")
                } else { 
                    b.partial_cmp(a).expect("Cannot compare values") 
                }
            })
        }

        // offset rows
        if let Some(offset) = select.offset {
            row_indexes.drain(0..offset.min(row_indexes.len()));
        }

        // limit rows
        if let Some(limit) = select.limit {
            row_indexes.truncate(limit);
        }

        // build result set
        let mut result_set = vec![];
        for row_index in row_indexes {
            let row = table.data.get(row_index).expect("Cannot get row with row_index");
            let mut result_row = vec![];

            // evaluate columns
            for (i, _) in &column_names {
                result_row.push(row.get(*i).unwrap().clone());
            }

            // set row variables to be used in special columns
            if special_columns.len() > 0 {
                let mut variables = self.variables.borrow_mut(); 
                for (name, i) in &column_map {
                    variables.insert(name.to_string(), row.get(*i).unwrap().clone());
                }
            }

            // evaluate special columns
            for (i, node) in &special_columns {
                let value = self.run(node)?.expect("Special column must return a value");
                result_row.insert(*i, value)
            }

            result_set.push(Value::Array(result_row));
        }

        Ok(Some(Value::Array(result_set)))
    }

    fn eval_insert(&self, insert: &InsertQuery) -> Result<Option<Value>, String> {
        // eval the key_values
        let mut key_values = vec![];
        for (key, value) in &insert.key_values {
            let value = self.run(value)?.expect(&format!("Value for column '{}' was not evaluated", key));
            key_values.push((key, value));
        }
        // TODO: check duplicates

        let mut database = self.database.borrow_mut();
        let table = database.get_table_mut(&insert.table).unwrap();
        let column_names = insert.key_values.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>();
        
        // check if columns exist in the table
        // table.check_columns_exist(&column_names)?;
        column_names.iter().map(|name| {
            table.check_column_exists(name)
        }).collect::<Result<_, _>>()?;

        // check if required columns are present
        let missing_columns = table.columns.iter().filter_map(|column| {
            if !column_names.contains(&column.name.as_ref()) 
                && column.not_null && column.default.is_none() {
                Some(column.name.clone())
            } else {
                None
            }
        }).collect::<Vec<_>>();

        if missing_columns.len() > 0 {
            return Err(format!("Missing required columns: {:?}", missing_columns));
        }

        // create row
        // let row = table.create_row(&key_values)?;
        let mut row = Row::new();
        for (i, column) in table.columns.iter().enumerate() {
            let value = match key_values.iter().find(|(key, _)| *key == &column.name) {
                Some((_, value)) => Some(value.clone()),
                None => match column.default {
                    Some(ref default) => {
                        let value = column.data_type.parse(default)?;  
                        Some(value) 
                    },
                    None => match column.not_null {
                        true => return Err(format!("Column '{}' does not allow NULL values", column.name)),
                        false => None,
                    }
                }
            };

            let value = value.unwrap_or(Value::Null);

            // TODO: fix this 
            // we have to use this way of converting back to str and then to value since thats the
            // old way of doing this in the previous query runner which should be deprecated
            let value_str = match value {
                Value::Null => None,
                v => Some(v.to_string())
            };

            let parsed_value = column.validate_option(&value_str)?;
            
            row.set(i, parsed_value)
        }

        // check if row passes all unique constraints
        table.check_unique(&row)?;

        let row_values = row.iter().map(|value| value.clone()).collect();
        table.data.buf_rows.push(row);
        table.sync_buffer()?;

        Ok(Some(Value::Array(vec![Value::Array(row_values)])))
    }
    
    fn eval_update(&self, update: &UpdateQuery) -> Result<Option<Value>, String> {
        todo!()
    }

    fn eval_delete(&self, delete: &DeleteQuery) -> Result<Option<Value>, String> {
        todo!()
    }
}
