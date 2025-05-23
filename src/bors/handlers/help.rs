use crate::bors::Comment;
use crate::bors::RepositoryState;
use crate::bors::command::{Approver, BorsCommand, RollupMode};
use crate::database::DelegatedPermission;
use crate::github::PullRequest;
use std::sync::Arc;

pub(super) async fn command_help(
    repo: Arc<RepositoryState>,
    pr: &PullRequest,
) -> anyhow::Result<()> {
    let help = [
        BorsCommand::Approve {
            approver: Approver::Myself,
            priority: None,
            rollup: None,
        },
        BorsCommand::Approve {
            approver: Approver::Specified("".to_string()),
            priority: None,
            rollup: None,
        },
        BorsCommand::Unapprove,
        BorsCommand::SetPriority(0),
        BorsCommand::SetDelegate(DelegatedPermission::Review),
        BorsCommand::Undelegate,
        BorsCommand::Try {
            parent: None,
            jobs: vec![],
        },
        BorsCommand::TryCancel,
        BorsCommand::SetRollupMode(RollupMode::Always),
        BorsCommand::Info,
        BorsCommand::Ping,
        BorsCommand::Help,
        BorsCommand::OpenTree,
        BorsCommand::TreeClosed(0),
    ]
    .into_iter()
    .map(|help| format!("- {}", get_command_help(help)))
    .collect::<Vec<_>>()
    .join("\n");

    repo.client
        .post_comment(pr.number, Comment::new(help))
        .await?;
    Ok(())
}

fn get_command_help(command: BorsCommand) -> String {
    // !!! When modifying this match, also update the command list above (in [`command_help`]) !!!
    let help = match command {
        BorsCommand::Approve {
            approver: Approver::Myself,
            ..
        } => {
            "`r+ [p=<priority>] [rollup=<never|iffy|maybe|always>]`: Approve this PR. Optionally, you can specify `<priority>`, `<rollup>`."
        }
        BorsCommand::Approve {
            approver: Approver::Specified(_),
            ..
        } => {
            "`r=<user> [p=<priority>]`: Approve this PR on behalf of `<user>`. Optionally, you can specify a `<priority>`."
        }
        BorsCommand::Unapprove => "`r-`: Unapprove this PR",
        BorsCommand::SetPriority(_) => "`p=<priority>`: Set the priority of this PR",
        BorsCommand::SetDelegate(_) => {
            "`delegate=<try|review>`: Delegate permissions to the PR author\n- `delegate+`: Delegate review permissions to the PR author"
        }
        BorsCommand::Undelegate => "`delegate-`: Remove any previously granted delegation",
        BorsCommand::Help => "`help`: Print this help message",
        BorsCommand::Ping => "`ping`: Check if the bot is alive",
        BorsCommand::Try { .. } => {
            "`try [parent=<parent>] [jobs=<jobs>]`: Start a try build. Optionally, you can specify a `<parent>` SHA or a list of `<jobs>` to run"
        }
        BorsCommand::TryCancel => "`try cancel`: Cancel a running try build",
        BorsCommand::SetRollupMode(_) => {
            "`rollup=<never|iffy|maybe|always>`: Mark the rollup status of the PR"
        }
        BorsCommand::Info => {
            "`info`: Get information about the current PR including delegation, priority, merge status, and try build status"
        }
        BorsCommand::OpenTree => "`treeclosed-`, `treeopen`: Open the repository tree for merging",
        BorsCommand::TreeClosed(_) => {
            "`treeclosed=<priority>`: Close the tree for PRs with priority less than `<priority>`"
        }
    };
    help.to_string()
}

#[cfg(test)]
mod tests {
    use crate::tests::mocks::run_test;

    #[sqlx::test]
    async fn help_command(pool: sqlx::PgPool) {
        run_test(pool, |mut tester| async {
            tester.post_comment("@bors help").await?;
            insta::assert_snapshot!(tester.get_comment().await?, @r"
            - `r+ [p=<priority>] [rollup=<never|iffy|maybe|always>]`: Approve this PR. Optionally, you can specify `<priority>`, `<rollup>`.
            - `r=<user> [p=<priority>]`: Approve this PR on behalf of `<user>`. Optionally, you can specify a `<priority>`.
            - `r-`: Unapprove this PR
            - `p=<priority>`: Set the priority of this PR
            - `delegate=<try|review>`: Delegate permissions to the PR author
            - `delegate+`: Delegate review permissions to the PR author
            - `delegate-`: Remove any previously granted delegation
            - `try [parent=<parent>] [jobs=<jobs>]`: Start a try build. Optionally, you can specify a `<parent>` SHA or a list of `<jobs>` to run
            - `try cancel`: Cancel a running try build
            - `rollup=<never|iffy|maybe|always>`: Mark the rollup status of the PR
            - `info`: Get information about the current PR including delegation, priority, merge status, and try build status
            - `ping`: Check if the bot is alive
            - `help`: Print this help message
            - `treeclosed-`, `treeopen`: Open the repository tree for merging
            - `treeclosed=<priority>`: Close the tree for PRs with priority less than `<priority>`
            ");
            Ok(tester)
        })
        .await;
    }
}
