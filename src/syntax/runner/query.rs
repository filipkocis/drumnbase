use crate::{syntax::{ast::{Query, InsertQuery, SelectQuery, UpdateQuery, DeleteQuery, Node, Literal, Operator, Expression}, context::{RunnerContextScope, RunnerContextFields}}, basics::{Value, Row, value::NumericValue}, auth::{Authorize, action::TableAction, RlsAction}};

use super::{Runner, Ctx, RunnerResult};

impl Runner {
    pub(super) fn eval_query(&self, query: &Query, ctx: &Ctx) -> RunnerResult {
        if ctx.is_schema() {
            return Err("Invalid schema, cannot run queries in schema context".to_string())
        }

        let ctx = &Ctx::scoped(ctx.clone());

        let result = match query {
            Query::Select(select) => self.eval_select(select, ctx),
            Query::Insert(insert) => self.eval_insert(insert, ctx),
            Query::Update(update) => self.eval_update(update, ctx),
            Query::Delete(delete) => self.eval_delete(delete, ctx),
        };

        result
    }

    fn eval_policies(&self, policies: &[&Node], ctx: &Ctx) -> Result<bool, String> {
        if policies.len() == 0 { 
            return Ok(true)
        }

        for policy in policies {
            match self.run(policy, ctx)? {
                Some(Value::Boolean(true)) => return Ok(true),
                Some(Value::Boolean(false)) => (),
                _ => return Err("Policy must return a boolean value".to_string())
            }
        }

        Ok(false)
    }

    fn eval_select(&self, select: &SelectQuery, ctx: &Ctx) -> RunnerResult {
        let database = self.database.read();
        let table = match database.get_table(&select.table) {
            Some(table) => table,
            None => return Err(format!("Table '{}' does not exist in database '{}'", select.table, database.name))
        };

        table.authorize(&ctx.cluster_user(), TableAction::Select)?;

        let column_map = table.get_column_map(&table.get_column_names()).unwrap();
        let ctx = &Ctx::scoped_with(ctx.clone(), column_map);

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
        let policies = table.police(&ctx.cluster_user(), RlsAction::Select);
        let mut row_indexes = vec![];
        for (i, row) in table.data.iter().enumerate() {
            if row.is_deleted() { continue }

            ctx.set_row(row);

            // check rls
            if !self.eval_policies(&policies, ctx)? {
                continue
            }

            let where_clause_result = match &select.where_clause {
                Some(node) => self.run(node, ctx),
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
        let result_row_capacity = column_names.len() + special_columns.len();
        let mut result_set = Vec::with_capacity(row_indexes.len());
        for row_index in row_indexes {
            let row = table.data.get(row_index).expect("Cannot get row with row_index");
            let mut result_row = Vec::with_capacity(result_row_capacity);

            // evaluate columns
            for (i, _) in &column_names {
                result_row.push(row.get(*i).unwrap().clone());
            }

            // set row variables to be used in special columns
            if special_columns.len() > 0 {
                ctx.set_row(row);
            }

            // evaluate special columns
            for (i, node) in &special_columns {
                let value = self.run(node, &ctx)?.expect("Special column must return a value");
                result_row.insert(*i, value)
            }

            result_set.push(Value::Array(result_row));
        }

        Ok(Some(Value::Array(result_set)))
    }

    fn eval_insert(&self, insert: &InsertQuery, ctx: &Ctx) -> RunnerResult {
        let mut database = self.database.write().map_err(|_| "Cannot call 'query insert' when in read mode")?;
        let table = match database.get_table_mut(&insert.table) {
            Some(table) => table,
            None => return Err(format!("Table '{}' does not exist in database '{}'", insert.table, database.name))
        };

        table.authorize(&ctx.cluster_user(), TableAction::Insert)?;

        // eval the key_values
        let mut key_values = vec![];
        for (key, value) in &insert.key_values {
            let value = self.run(value, ctx)?.expect(&format!("Value for column '{}' was not evaluated", key));
            key_values.push((key, value));
        }
        // TODO: check duplicates

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
                Some((_, value)) => value.clone(),
                None => match column.default {
                    Some(ref default) => {
                        let value = column.data_type.parse(default)?;  
                        value
                    },
                    None => match column.not_null {
                        true => return Err(format!("Column '{}' does not allow NULL values", column.name)),
                        false => Value::Null,
                    }
                }
            };

            let parsed_value = column.transform_value(&value)?;
            row.set(i, parsed_value)
        }

        // check if row passes all unique constraints
        table.check_unique(&row)?;

        // check rls
        let column_map = table.get_column_map(&table.get_column_names()).unwrap();
        let ctx = &Ctx::scoped_with(ctx.clone(), column_map);
        ctx.set_row(&row);
        let policies = table.police(&ctx.cluster_user(), RlsAction::Insert);
        if !self.eval_policies(&policies, ctx)? {
            return Err("Insertion violates row level security policy".to_string())
        }

        let row_values = row.iter().map(|value| value.clone()).collect();
        table.data.buf_rows.push(row);
        table.sync_buffer()?;

        Ok(Some(Value::Array(vec![Value::Array(row_values)])))
    }
    
    fn eval_update(&self, update: &UpdateQuery, ctx: &Ctx) -> RunnerResult {
        let mut database = self.database.write().map_err(|_| "Cannot call 'query update' when in read mode")?;
        let table = match database.get_table_mut(&update.table) {
            Some(table) => table,
            None => return Err(format!("Table '{}' does not exist in database '{}'", update.table, database.name))
        };

        table.authorize(&ctx.cluster_user(), TableAction::Update)?;

        // eval the key_values
        let mut key_values = vec![];
        for (key, value) in &update.key_values {
            let value = self.run(value, ctx)?.expect(&format!("Value for column '{}' was not evaluated", key));
            key_values.push((key, value));
        }
        // TODO: check duplicates

        let column_names = update.key_values.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>();
        let column_map = table.get_column_map(&table.get_column_names()).unwrap();
        let ctx = &Ctx::scoped_with(ctx.clone(), column_map);

        // check if columns exist in the table
        // table.check_columns_exist(&column_names)?;
        column_names.iter().map(|name| {
            table.check_column_exists(name)
        }).collect::<Result<_, _>>()?;

        // check if any of the columns have unique constraints
        let unique_columns = column_names.iter().filter(|name| {
            table.get_column(name).unwrap().unique
        }).collect::<Vec<_>>();

        if unique_columns.len() > 0 {
            // TODO: implement 'limit single'
            return Err(format!("Columns '{:?}' have unique constraint, to update such columns, use 'limit single'", unique_columns))
        }

        // TODO: get_parsed_key_vals, this is the manual way
        let mut parsed_key_vals = vec![];
        for (name, value) in &key_values {
            let i = table.get_column_index(name).unwrap();
            let value_str = match value {
                Value::Null => None,
                v => Some(v.to_string())
            };

            let parsed_value = table.columns[i].validate_option(&value_str)?;
            parsed_key_vals.push((i, parsed_value));
        }
        let column_indexes = parsed_key_vals.iter().map(|(i, _)| *i).collect::<Vec<_>>();

        // HINT: this is somewhat safe, because policies will not be modified via the mutable
        // usage of the table below, and it's faster than cloning
        let policies = table.police(&ctx.cluster_user(), RlsAction::Update).iter().map(|&p| p as *const Node ).collect::<Vec<_>>();
        let policies = policies.into_iter().map(|p| unsafe { &*p as &Node }).collect::<Vec<_>>();

        // evaluate where clause on each row
        let mut updated_rows_count = 0;
        for index in 0..table.data.len() {
            let row = table.data.get_mut(index).unwrap();
            if row.is_deleted() { continue }

            ctx.set_row(row);

            // check rls
            // TODO: give access to "new row" in ctx for rls
            if !self.eval_policies(&policies, ctx)? {
                return Err("Insertion violates row level security policy".to_string())
            }

            let where_clause_result = match &update.where_clause {
                Some(node) => self.run(node, ctx),
                // None => Ok(Some(Value::Boolean(true)))
                None => return Err("Update query must have a where clause".to_string())
            };

            match where_clause_result {
                Ok(Some(Value::Boolean(true))) => {
                    row.update_with(&parsed_key_vals);
                    table.sync_row_parts(index, &column_indexes)?;                           
                    updated_rows_count += 1;
                },
                Ok(Some(Value::Boolean(false))) => (),
                Ok(_) => return Err("Where clause must return a boolean value".to_string()),
                Err(err) => return Err(err)
            };
        }
        
        Ok(Some(Value::Numeric(NumericValue::IntU64(updated_rows_count as u64))))
    }

    fn eval_delete(&self, delete: &DeleteQuery, ctx: &Ctx) -> RunnerResult {
        let mut database = self.database.write().map_err(|_| "Cannot call 'query delete' when in read mode")?;
        let table = match database.get_table_mut(&delete.table) {
            Some(table) => table,
            None => return Err(format!("Table '{}' does not exist in database '{}'", delete.table, database.name))
        };

        table.authorize(&ctx.cluster_user(), TableAction::Delete)?;

        let column_map = table.get_column_map(&table.get_column_names()).unwrap();
        let ctx = &Ctx::scoped_with(ctx.clone(), column_map);

        // HINT: this is somewhat safe, because policies will not be modified via the mutable
        // usage of the table below, and it's faster than cloning
        let policies = table.police(&ctx.cluster_user(), RlsAction::Delete).iter().map(|&p| p as *const Node ).collect::<Vec<_>>();
        let policies = policies.into_iter().map(|p| unsafe { &*p as &Node }).collect::<Vec<_>>();

        // evaluate where clause on each row
        let mut deleted_rows_count = 0;
        for index in 0..table.data.len() {
            let row = table.data.get_mut(index).unwrap();
            if row.is_deleted() { continue }
            
            ctx.set_row(row); 

            // check rls
            if !self.eval_policies(&policies, ctx)? {
                return Err("Insertion violates row level security policy".to_string())
            }

            let where_clause_result = match &delete.where_clause {
                Some(node) => self.run(node, ctx),
                // None => Ok(Some(Value::Boolean(false)))
                None => return Err("Delete query must have a where clause".to_string())
            };

            match where_clause_result {
                Ok(Some(Value::Boolean(true))) => {
                    row.mark_deleted();
                    table.sync_flags(index)?;
                    deleted_rows_count += 1;
                },
                Ok(Some(Value::Boolean(false))) => (),
                Ok(v) => return Err(format!("Where clause must return a boolean value, got: {:?}", v)),
                Err(err) => return Err(err)
            };
        }
        
        Ok(Some(Value::Numeric(NumericValue::IntU64(deleted_rows_count as u64))))
    }
}
