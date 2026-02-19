mod create;
mod delete;
mod download;
mod get;
mod list;
mod update;

use crate::adapter::cli::{CreateArgs, DeleteArgs, DownloadArgs, GetArgs, ListArgs, UpdateArgs};
use crate::application::ApplicationState;

pub(super) async fn handle_get(state: &ApplicationState, args: GetArgs) -> anyhow::Result<()> {
    get::handle_get(state, args).await
}

pub(super) async fn handle_list(state: &ApplicationState, args: ListArgs) -> anyhow::Result<()> {
    list::handle_list(state, args).await
}

pub(super) async fn handle_create(
    state: &ApplicationState,
    args: CreateArgs,
) -> anyhow::Result<()> {
    create::handle_create(state, args).await
}

pub(super) async fn handle_update(
    state: &ApplicationState,
    args: UpdateArgs,
) -> anyhow::Result<()> {
    update::handle_update(state, args).await
}

pub(super) async fn handle_delete(
    state: &ApplicationState,
    args: DeleteArgs,
) -> anyhow::Result<()> {
    delete::handle_delete(state, args).await
}

pub(super) async fn handle_download(
    state: &ApplicationState,
    args: DownloadArgs,
) -> anyhow::Result<()> {
    download::handle_download(state, args).await
}
