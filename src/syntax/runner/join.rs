use std::{ptr, collections::HashSet};

use crate::{basics::{Table, Value, Row}, syntax::{context::Ctx, ast::Node}};

use super::Runner;

impl Runner {
    pub fn perform_joins(&self, tables: &Vec<Table>, join: Vec<(String, Node)>, ctx: &Ctx) -> Result<UnsafeJoinedTables, String> {
        let mut result = UnsafeJoinedTables::from_table(&tables[0]);
        println!("init len {:?}", result.data.len());

        for i in 1..tables.len() {
            let current_table = &tables[i];
            let join_type = &join[i - 1].0;
            let join_condition = &join[i - 1].1;

            result = self.apply_join(result, current_table, join_type, join_condition, ctx)?;
            println!("after {} len {:?}", i, result.data.len());
        }

        Ok(result)
    }

    pub fn apply_join(&self, table_a: UnsafeJoinedTables, table_b: &Table, join_type: &str, condition: &Node, ctx: &Ctx) -> Result<UnsafeJoinedTables, String> {
        let mut output_table = UnsafeJoinedTables::new();

        match join_type {
            "INNER" | "LEFT" | "RIGHT" | "FULL" => (),
            _ => return Err(format!("Join type '{}' not supported", join_type))
        }

        let mut matched_b_rows = HashSet::new();

        for row_a in table_a.data.iter() {
            let mut match_found = false;
            for row_b in table_b.data.iter() {
                let condition_result = self.run(condition, &ctx)?;
                if !matches!(condition_result, Some(Value::Boolean(true))) {
                    continue
                }

                let mut combined_row = row_a.clone();
                combined_row.push(row_b);

                output_table.data.push(combined_row);
                match_found = true;

                if join_type == "FULL" || join_type == "RIGHT" {
                    matched_b_rows.insert(row_b as *const Row);
                }
            }

            if !match_found && (join_type == "LEFT" || join_type == "FULL") {
                let mut combined_row = row_a.clone();
                combined_row.push(ptr::null());
                output_table.data.push(combined_row);
            }
        }

        if join_type == "RIGHT" || join_type == "FULL" {
            for row_b in table_b.data.iter() {
                if matched_b_rows.contains(&(row_b as *const Row)) {
                    continue;
                }

                let mut combined_row = vec![ptr::null(); table_a.tables.len()];
                combined_row.push(row_b);
                output_table.data.push(combined_row);
            }
        }

        output_table.tables = table_a.tables;
        output_table.tables.push(table_b as *const Table);

        return Ok(output_table)
    }



        
    //     match join_type {
    //         "INNER" => {
    //             for row_a in table_a.data.iter() {
    //                 for row_b in table_b.data.iter() {
    //                     let condition_result = self.run(condition, &ctx)?;
    //                     if !matches!(condition_result, Some(Value::Boolean(true))) {
    //                         continue
    //                     }
    //
    //                     let mut combined_row = row_a.clone();
    //                     combined_row.push(row_b);
    //
    //                     output_table.data.push(combined_row);
    //                 }
    //             }
    //         },
    //         "LEFT" => {
    //             for row_a in table_a.data.iter() {
    //                 let mut match_found = false;
    //                 for row_b in table_b.data.iter() {
    //                     let condition_result = self.run(condition, &ctx)?;
    //                     if !matches!(condition_result, Some(Value::Boolean(true))) {
    //                         continue
    //                     }
    //
    //                     let mut combined_row = row_a.clone();
    //                     combined_row.push(row_b);
    //                     
    //                     output_table.data.push(combined_row);
    //                     match_found = true;
    //                 }
    //
    //                 if !match_found {
    //                     let mut combined_row = row_a.clone();
    //                     combined_row.push(ptr::null());
    //                     output_table.data.push(combined_row);
    //                 }
    //             }
    //         },
    //         "RIGHT" => {
    //             for row_b in table_b.data.iter() {
    //                 let mut match_found = false;
    //                 for row_a in table_a.data.iter() {
    //                     let condition_result = self.run(condition, &ctx)?;
    //                     if !matches!(condition_result, Some(Value::Boolean(true))) {
    //                         continue
    //                     }
    //
    //                     let mut combined_row = row_a.clone();
    //                     combined_row.push(row_b);
    //                     
    //                     output_table.data.push(combined_row);
    //                     match_found = true;
    //                 }
    //
    //                 if !match_found {
    //                     let mut combined_row = vec![ptr::null(); table_a.tables.len()];
    //                     combined_row.push(row_b);
    //                     output_table.data.push(combined_row);
    //                 }
    //             }
    //
    //         },
    //         "FULL" => {
    //             let mut matched_b_rows = HashSet::new();
    //
    //             for row_a in table_a.data.iter() {
    //                 let mut match_found = false;
    //                 for row_b in table_b.data.iter() {
    //                     let condition_result = self.run(condition, &ctx)?;
    //                     if !matches!(condition_result, Some(Value::Boolean(true))) {
    //                         continue
    //                     }
    //
    //                     let mut combined_row = row_a.clone();
    //                     combined_row.push(row_b);
    //                     
    //                     output_table.data.push(combined_row);
    //                     match_found = true;
    //                     matched_b_rows.insert(row_b as *const Row);
    //                 }
    //
    //                 if !match_found {
    //                     let mut combined_row = row_a.clone();
    //                     combined_row.push(ptr::null());
    //                     output_table.data.push(combined_row);
    //                 }
    //             }
    //
    //             for row_b in table_b.data.iter() {
    //                 if matched_b_rows.contains(&(row_b as *const Row)) {
    //                     continue;
    //                 }
    //
    //                 let mut combined_row = vec![ptr::null(); table_a.tables.len()];
    //                 combined_row.push(row_b);
    //                 output_table.data.push(combined_row);
    //             }
    //
    //         },
    //         _ => return Err(format!("Join type '{}' not supported", join_type))
    //     } 
    //
    //     output_table.tables = table_a.tables;
    //     output_table.tables.push(table_b as *const Table);
    //
    //     Ok(output_table)
    // }
}

pub struct UnsafeJoinedTables {
    // [table_a, table_b, table_c] -> used to index into the correct "row" inside the data row 
    // data[0][table_a_index] -> row_table_a
    pub tables: Vec<*const Table>,
    // [row_table_a, row_table_b, row_table_b] -> makes one joined row -> data[0]
    pub data: Vec<Vec<*const Row>>,
}

impl UnsafeJoinedTables {
    fn new() -> Self {
        Self {
            tables: vec![],
            data: vec![],
        }
    }

    fn from_table(table: &Table) -> Self {
        let mut join_table = Self::new();
        join_table.tables.push(table as *const Table);

        let mut rows = vec![];
        for row in table.data.iter() {
            rows.push(vec![row as *const Row]);
        }
        join_table.data = rows;

        join_table
    }
}
