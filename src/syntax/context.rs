use std::{rc::Rc, cell::RefCell, collections::HashMap, borrow::Cow, sync::{Arc, RwLock}, ptr};

use crate::{basics::{Row, Value, Table}, auth::User, database::RunOptions, cluster::Cluster};

pub type Ctx<'a> = Rc<RunnerContext<'a>>;
type Scope<'a> = Rc<RefCell<HashMap<String, ScopeValue<'a>>>>;
type ScopeValue<'a> = Rc<RefCell<Cow<'a, Value>>>;

/// Struct to hold the context of a runner, it is passed around to all runner functions
pub struct RunnerContext<'a> {
    /// Variables in the current scope
    variables: Scope<'a>,
    /// The current row being processed by the query runner
    ///
    /// Pointer will be valid as long as the context isn't stored anywhere. Since the row is
    /// borrowed from read lock on the database and context is scoped to that function call,
    /// nothing can access the row in the meantime.
    current_row: RefCell<Option<*const Row>>,
    current_column_map: Option<HashMap<String, usize>>,

    // row representation of 'joined rows'
    pub current_unsafe_row: RefCell<*const Vec<*const Row>>,
    // key: (table_name, column_name) -> (table_index, column_index)
    pub current_unsafe_column_map: RefCell<HashMap<(String, String), (usize, usize)>>,

    options: Rc<RunOptions>, // rc for easier cloning when scoping
    pub parent: Option<Ctx<'a>>,
}

pub trait RunnerContextScope<'a> {
    /// Create a new context with the current context as the parent
    fn scoped(parent: Ctx<'a>) -> Self;
    fn scoped_with(parent: Ctx<'a>, column_map: HashMap<String, usize>) -> Self; 
}
impl<'a> RunnerContextScope<'a> for RunnerContext<'a> {
    fn scoped(parent: Ctx<'a>) -> Self {
        let mut ctx = RunnerContext::new(parent.options.clone());
        ctx.parent = Some(parent.clone());
        ctx
    }

    fn scoped_with(parent: Ctx<'a>, column_map: HashMap<String, usize>) -> Self {
        let mut scoped = Self::scoped(parent); 
        scoped.set_column_map(column_map);
        scoped
    }
}
impl<'a> RunnerContextScope<'a> for Ctx<'a> {
    fn scoped(parent: Ctx<'a>) -> Ctx {
        let mut ctx = RunnerContext::new(parent.options.clone());
        ctx.parent = Some(parent.clone());
        Rc::new(ctx)
    }

    fn scoped_with(parent: Ctx<'a>, column_map: HashMap<String, usize>) -> Self {
        let mut scoped = RunnerContext::scoped(parent); 
        scoped.set_column_map(column_map);
        Rc::new(scoped)
    }
}

pub trait RunnerContextVariable<'a> {
    /// Get the nearest variable 'name', or the nearest column if in a row context (row takes
    /// precedence over variables in a scope)
    ///
    /// Errors if the variable is not found
    fn get(&self, name: &str) -> Result<ScopeValue<'a>, String>;
    /// Assign value to nearest variable 'name'
    ///
    /// Errors if the variable is not found
    fn assign(&self, name: &str, value: Value) -> Result<(), String>;
    /// Declare (or shadow) a new variable 'name' in the current scope
    fn declare(&self, name: &str, value: Value);
    /// Drop the variable 'name' from the current scope
    ///
    /// Errors if the variable is not found in the current scope
    fn drop(&self, name: &str) -> Result<(), String>;
}

impl<'a> RunnerContextVariable<'a> for Ctx<'a> {
    fn get(&self, name: &str) -> Result<ScopeValue<'a>, String> {
        let mut current = Some(self);

        while let Some(ctx) = current {
            // check if variable is a column in the current row
            let column_map = ctx.column_map();
            let row = ctx.row();
            if column_map.is_some() && row.is_some() {
                let row = unsafe { &*row.unwrap() };

                if let Some(index) = column_map.unwrap().get(name) {
                    let value = row.get(*index).unwrap();
                    let wrapped = Rc::new(RefCell::new(Cow::Borrowed(value)));
                    return Ok(wrapped)
                }
            }

            // check if variable exists in this scope
            if let Some(v) = ctx.variables.borrow().get(name) {
                return Ok(v.clone())
            }

            // move to the parent context
            current = ctx.parent.as_ref();
        }

        Err(format!("Variable '{}' not found", name))
    }

    fn assign(&self, name: &str, value: Value) -> Result<(), String> {
        match self.get(name) {
            Ok(v) => *v.borrow_mut() = Cow::Owned(value),
            Err(e) => Err(e)?,
        }

        Ok(())
    }

    fn declare(&self, name: &str, value: Value) {
        let name = name.to_string();
        let value = Rc::new(RefCell::new(Cow::Owned(value)));
        self.variables.borrow_mut().insert(name, value);    
    }

    fn drop(&self, name: &str) -> Result<(), String> {
        if self.variables.borrow_mut().remove(name).is_none() {
            Err(format!("Variable '{}' not found in current scope", name))?
        }

        Ok(())
    }
}

pub trait RunnerContextFields<'a> {
    fn row(&self) -> Option<*const Row>;
    fn column_map(&self) -> Option<&HashMap<String, usize>>;
    fn set_row(&self, row: &Row);
    fn set_column_map(&mut self, column_map: HashMap<String, usize>);
}

impl<'a> RunnerContextFields<'a> for RunnerContext<'a> {
    fn row(&self) -> Option<*const Row> {
        *self.current_row.borrow()
    }

    fn column_map(&self) -> Option<&HashMap<String, usize>> {
        self.current_column_map.as_ref()
    }

    fn set_row(&self, row: &Row) {
        *self.current_row.borrow_mut() = Some(row); 
    }

    fn set_column_map(&mut self, column_map: HashMap<String, usize>) {
        self.current_column_map = Some(column_map); 
    }
}

impl<'a> RunnerContext<'a> {
    pub fn new(options: Rc<RunOptions>) -> Self {
        Self {
            variables: Rc::new(RefCell::new(HashMap::new())),
            current_row: RefCell::new(None),
            current_column_map: None,
            options,
            parent: None,

            current_unsafe_row: RefCell::new(ptr::null()),
            current_unsafe_column_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_ctx(options: Rc<RunOptions>) -> Ctx<'a> {
        Rc::new(RunnerContext::new(options))
    }

    pub fn cluster_user(&self) -> &Rc<User> {
        &self.options.cluster_user
    }

    pub fn auth_user(&self) -> &Rc<User> {
        &self.options.auth_user
    }

    pub fn cluster(&self) -> &Arc<RwLock<Cluster>> {
        &self.options.cluster
    }

    pub fn is_schema(&self) -> bool {
        self.options.is_schema
    }
}

impl<'a> RunnerContext<'a> {
    /// Get the row value based on member access in select query 
    pub fn get_from(&self, table: &str, column: &str) -> Result<&Value, String> {
        let map = self.current_unsafe_column_map.borrow();
        let key = (table.to_string(), column.to_string());

        if let Some((table_index, column_index)) = map.get(&key) {
            let unsafe_joined_row = *self.current_unsafe_row.borrow();
            if unsafe_joined_row.is_null() {
                return Err("No joined row found".to_string())
            }

            let joined_row = unsafe { &*unsafe_joined_row };
            let unsafe_row = joined_row[*table_index];
            if unsafe_row.is_null() {
                return Ok(&Value::Null)
            }

            let row = unsafe { &*unsafe_row };
            let value = row.get(*column_index).unwrap();

            return Ok(value)
        }

        Err(format!("Column '{}' not found in table '{}'", column, table))
    }

    pub fn set_joined_row(&self, row: &Vec<*const Row>) {
        *self.current_unsafe_row.borrow_mut() = row;
    }

    pub fn set_joined_tables(&self, tables: &Vec<*const Table>) {
        let mut map = HashMap::new();

        for (table_index, table) in tables.iter().enumerate() {
            let table = unsafe { &*(*table) };

            for (column_index, column) in table.columns.iter().enumerate() {
                map.insert((table.name.clone(), column.name.clone()), (table_index, column_index));
            }
        }

        *self.current_unsafe_column_map.borrow_mut() = map;
    }
}
