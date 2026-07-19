use crate::{context::AppContext, error::AppError};

pub async fn create_issue(ctx: AppContext) -> Result<(), AppError> {
    let jira_projects = ctx.vault.list_issue_keys()?;

    for project in jira_projects {
        println!("{}", project);
    }

    todo!()
}
