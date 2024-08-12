use crate::{database::database::Database, query::query::QueryType, file::{data::LoadMode, write::DatabaseWriter}, basics::row::Row};

use super::{parser::{SimpleQueryParser, QueryParser}, query::{Query, QueryResult, InsertQuery, UpdateQuery, DeleteQuery, SelectQuery}};

pub trait QueryRunner {
    fn run_query(&mut self, query: &str) -> Result<(), String>;
}

impl QueryRunner for Database {
    fn run_query(&mut self, query: &str) -> Result<(), String> {
        let query = query.trim();
        if query.is_empty() { return Ok(()); }
        
        let query = SimpleQueryParser::from(query)?.parse()?;
        println!("{:#?}", query);
        let result = query.apply_to(self)?;

        println!();
        println!("AMOUNT: {:#?}", result.amount);
        for row in result.data.iter() {
            println!("ROW: {}", row);
        }
        println!();

        Ok(())
    }
}

impl Query {
    fn apply_to(&self, database: &mut Database) -> Result<QueryResult, String> {
        let table = self.get_table_name();
        let query = self.get_specific().ok_or("Bad query")?;

        // temporary check
        if database.get_table(table).ok_or(format!("Table '{}' not found", table))?.data.load_mode == LoadMode::Disk {
            return Err(format!("Table '{}' is in LoadMode::Disk, which is not supported yet", table));
        };

        match query {
            QueryType::Select(select) => database.select(table, select),
            QueryType::Insert(insert) => database.insert(table, insert),
            QueryType::Update(update) => database.update(table, update),
            QueryType::Delete(delete) => database.delete(table, delete),
        }
    }
}

impl Database {
    fn select(&mut self, table: &str, select: &SelectQuery) -> Result<QueryResult, String> {
        let table = self.get_table(table).ok_or(format!("Table '{}' not found", table))?;

        let contains_star = select.columns.contains(&"*".to_string());
        let query_columns = match contains_star {
            true => table.get_column_names(),
            false => select.columns.clone(),
        };

        table.check_columns_exist(&select.columns)?;
         
        let where_chain = select.get_where();
        let mut checked_rows: Vec<&Row> = match where_chain {
            Some(chain) => {
                // TODO: if where clause has columns which are not selected, it errors, FIXME
                let index_map = table.get_column_map(&query_columns)?;
                let mut checked_rows = Vec::new();
                let parsed_chain = chain.get_parsed_value_chain(&table.columns)?;

                for row in table.data.iter() {
                    match parsed_chain.check(row, &index_map) {
                        Ok(true) => checked_rows.push(row),
                        Ok(false) => continue,
                        Err(e) => return Err(e),
                    }
                }

                checked_rows
            },
            None => table.data.iter().collect(),
        };

        let order = select.get_order();
        let sorted_rows = match order {
            Some(order) => {
                let column_index = table.get_column_index(&order.get_column())?;
                checked_rows.sort_by(|a, b| order.compare(a, b, column_index));
                checked_rows
            }
            None => checked_rows
        };

        const DEFAULT_LIMIT: usize = 1_000;
        let limit = select.get_limit().unwrap_or(DEFAULT_LIMIT);
        let offset = select.get_offset().unwrap_or(0);
        let limit_offset_rows = sorted_rows.into_iter().skip(offset).take(limit).collect::<Vec<_>>();

        let exclude = select.get_exclude();
        let keep_indexes = match exclude {
            Some(exclude_columns) => {
                let exclude_indexes = table.get_column_indexes(&exclude_columns)?;
                let keep_indexes = table.get_column_indexes(&query_columns)?;
                let indexes = keep_indexes
                    .into_iter()
                    .filter(|e| !exclude_indexes.contains(e))
                    .collect::<Vec<_>>();
                indexes
            },
            None => table.get_column_indexes(&query_columns)?,
        };


        let selected_rows = limit_offset_rows
            .into_iter()
            .map(|row| {
                row.with_kept_columns(&keep_indexes)
            })
            .collect::<Vec<_>>();

        Ok(QueryResult::from(selected_rows))
    }
        
    // TODO: implement UNIQUE constraint
    fn insert(&mut self, table: &str, insert: &InsertQuery) -> Result<QueryResult, String> {
        let table = self.get_table_mut(table).ok_or(format!("Table '{}' not found", table))?;
        let query_columns = insert.get_keys();

        // check if all columns from the query exist
        table.check_columns_exist(&query_columns)?;

        // check if all needed columns are present, if not, return error
        let missing_columns = table.columns
            .iter()
            .filter(|&column| 
                !query_columns.contains(&column.name) && column.not_null && column.default.is_none(),
            )
            .map(|column| column.name.clone())
            .collect::<Vec<_>>();

        if !missing_columns.is_empty() {
            return Err(format!("Missing columns: {:?}", missing_columns))
        }

        let new_row = table.create_row(&insert.key_vals)?; 

        // add new row to the table buffer, then write to disk/memory
        table.data.buf_rows.push(new_row.clone());
        table.write()?;

        Ok(QueryResult::from(vec![new_row]))
    }

    fn update(&mut self, table: &str, update: &UpdateQuery) -> Result<QueryResult, String> {
        todo!()
    }

    fn delete(&mut self, table: &str, delete: &DeleteQuery) -> Result<QueryResult, String> {
        todo!()
    }
}
