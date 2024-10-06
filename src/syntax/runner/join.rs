use std::{ptr, collections::HashSet};

use crate::{basics::{Table, Value, Row}, syntax::{context::{Ctx, RunnerContextFields, RunnerContextScope}, ast::{Node, Join, JoinType}}, auth::{RlsAction, action::TableAction, Authorize}};

use super::Runner;

impl Runner {
    /// Executes join operations on a base table
    ///
    /// Returns a joined table which should be used in where clause filtering
    pub fn perform_joins(&self, base_table: &Table, joins: &Vec<Join>, ctx: &Ctx) -> Result<UnsafeJoinedTables, String> {
        let database = self.database.read();
        let mut result = self.transform_table_into_joined(base_table, ctx)?;

        // authorize base table
        base_table.authorize(&ctx.cluster_user(), TableAction::Select)?;

        // check if all tables in joins exist, and authorize them
        for join in joins {
            let table = match database.get_table(&join.table) {
                Some(table) => table,
                None => return Err(format!("Table '{}' not found", join.table))
            };
            table.authorize(&ctx.cluster_user(), TableAction::Select)?;
        }

        // apply joins sequentially
        for join in joins {
            let current_table = database.get_table(&join.table).expect("Table should exist");

            result = self.apply_join(result, current_table, &join.join_type, &join.on, ctx)?;
        }

        Ok(result)
    }

    fn apply_join(&self, table_a: UnsafeJoinedTables, table_b: &Table, join_type: &JoinType, on: &Node, ctx: &Ctx) -> Result<UnsafeJoinedTables, String> {
        let mut output_table = UnsafeJoinedTables::new();
        let mut matched_b_rows = HashSet::new();
        let mut table_b_data = vec![];

        let column_map = table_b.get_column_map(&table_b.get_column_names()).unwrap();
        let ctx = &Ctx::scoped_with(ctx.clone(), column_map);
        ctx.set_joined_tables(&table_a.tables);
        let policies = table_b.police(&ctx.cluster_user(), RlsAction::Select);

        for row_a in table_a.data.iter() {
            ctx.set_joined_row(row_a);

            let mut match_found = false;
            for row_b in table_b.data.iter() {
                if row_b.is_deleted() { continue }

                // check rls
                ctx.set_row(row_b);
                if !self.eval_policies(&policies, ctx)? {
                    continue
                }

                if *join_type == JoinType::Right || *join_type == JoinType::Full {
                    // Push rows which have passed all checks (rls, deleted), so that we do not
                    // need to check them again in the bottom loop
                    table_b_data.push(row_b);
                }

                // check if the join condition is true
                match self.run(on, &ctx)? {
                    Some(Value::Boolean(true)) => (),
                    Some(Value::Boolean(false)) => continue,
                    _ => return Err("Join condition must return a boolean value".to_string()),
                };

                let mut combined_row = row_a.clone();
                combined_row.push(row_b);

                output_table.data.push(combined_row);
                match_found = true;

                if *join_type == JoinType::Full || *join_type == JoinType::Right {
                    matched_b_rows.insert(row_b as *const Row);
                }
            }

            if !match_found && (*join_type == JoinType::Left || *join_type == JoinType::Full) {
                let mut combined_row = row_a.clone();
                combined_row.push(ptr::null());
                output_table.data.push(combined_row);
            }
        }

        if *join_type == JoinType::Right || *join_type == JoinType::Full {
            for row_b in table_b_data {
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
}

impl Runner {
    /// Trasforms a table into a valid joined table
    ///
    /// # Usage
    /// Returned table is used to perform joins on with other tables
    ///
    /// # Note
    /// It performs checks for RLS policies and skips deleted rows
    fn transform_table_into_joined(&self, table: &Table, ctx: &Ctx) -> Result<UnsafeJoinedTables, String> {
        let mut join_table = UnsafeJoinedTables::new();
        join_table.tables.push(table as *const Table);
        let mut rows = vec![];

        let column_map = table.get_column_map(&table.get_column_names()).unwrap();
        let ctx = &Ctx::scoped_with(ctx.clone(), column_map);
        let policies = table.police(&ctx.cluster_user(), RlsAction::Select);

        for row in table.data.iter() {
            if row.is_deleted() { continue }

            // check rls
            ctx.set_row(row);
            if !self.eval_policies(&policies, ctx)? {
                continue
            }

            rows.push(vec![row as *const Row]);
        }

        join_table.data = rows;
        Ok(join_table)
    }
}
