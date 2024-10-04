use crate::{syntax::{context::Ctx, ast::{SDL, CreateSDL, GrantSDL}}, basics::{Column, Table}, auth::RlsPolicy};

use super::{Runner, RunnerResult};

impl Runner {
    pub(super) fn eval_sdl(&self, sdl: &SDL, ctx: &Ctx) -> RunnerResult {
        match sdl {
            SDL::Create(create) => self.eval_create(create, ctx),
            // SDL::Drop(drop) => self.eval_drop(drop, ctx),
            SDL::Grant(grant) => self.eval_grant(grant, ctx),
            _ => todo!()
        }
    }

    fn eval_create(&self, create: &CreateSDL, ctx: &Ctx) -> RunnerResult {
        match create {
            CreateSDL::Database { name } => self.eval_create_database(name, ctx),
            CreateSDL::Table { name, columns } => self.eval_create_table(name, columns, ctx),
            CreateSDL::RlsPolicy { table, policy } => self.eval_create_rls_policy(table, policy, ctx),
            CreateSDL::Role { name } => self.eval_create_role(name, ctx),
            CreateSDL::User { name, password, is_superuser } => self.eval_create_user(name, password, *is_superuser, ctx)
        }
    }

    fn eval_create_rls_policy(&self, table: &str, policy: &RlsPolicy, ctx: &Ctx) -> RunnerResult {
        if !ctx.cluster_user().is_superuser {
            return Err("Can't create rls policies, permission denied".to_string());
        }

        let mut database = self.database.write().map_err(|_| "Can't create rls when in read mode")?;
        database.create_rls_policy(table, policy.clone())?;

        Ok(None)
    }

    fn eval_create_database(&self, name: &str, ctx: &Ctx) -> RunnerResult {
        if !ctx.cluster_user().is_superuser {
            return Err("Can't create database, permission denied".to_string());
        }

        let mut cluster = ctx.cluster().write().map_err(|_| "Can't create database when in read mode")?;
        cluster.create_database(name)?;

        Ok(None)
    }

    fn eval_create_table(&self, name: &str, columns: &[Column], ctx: &Ctx) -> RunnerResult {
        if !ctx.cluster_user().is_superuser {
            return Err("Can't create table, permission denied".to_string());
        }

        let mut table = Table::new(name);
        table.columns = columns.to_vec();

        let mut database = self.database.write().map_err(|_| "Can't create table when in read mode")?;
        database.create_table(table)?;
        Ok(None)
    }

    fn eval_create_role(&self, name: &str, ctx: &Ctx) -> RunnerResult {
        if !ctx.cluster_user().is_superuser {
            return Err("Can't create role, permission denied".to_string());
        }

        let mut cluster = ctx.cluster().write().map_err(|_| "Can't create role when in read mode")?;
        cluster.create_role(name)?;

        Ok(None)
    }

    fn eval_create_user(&self, name: &str, password: &str, is_superuser: bool, ctx: &Ctx) -> RunnerResult {
        if !ctx.cluster_user().is_superuser {
            return Err("Can't create user, permission denied".to_string());
        }

        let mut cluster = ctx.cluster().write().map_err(|_| "Can't create user when in read mode")?;
        cluster.create_user(name, password, is_superuser)?;

        Ok(None)
    }

    fn eval_grant(&self, grant: &GrantSDL, ctx: &Ctx) -> RunnerResult {
        match grant {
            GrantSDL::Role { name, to } => self.eval_grant_role(name, to, ctx),
            // GrantSDL::Action { object, object_name, actions, table, to } => self.eval_grant_action(object, object_name, actions, table, to, ctx)
            _ => Err("Grant not implemented".to_string())
        }
    }

    fn eval_grant_role(&self, name: &str, to: &str, ctx: &Ctx) -> RunnerResult {
        if !ctx.cluster_user().is_superuser {
            return Err("Can't grant role, permission denied".to_string())
        }

        let mut cluster = ctx.cluster().write().map_err(|_| "Can't grant role when in read mode")?;
        cluster.grant_role(name, to)?;

        Ok(None)
    }
}
