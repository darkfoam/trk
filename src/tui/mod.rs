pub mod app;
pub mod input;
pub mod picker;

use crate::ctx::Ctx;
use crate::error::TrkError;

pub fn run(ctx: &Ctx, focus: bool) -> Result<(), TrkError> {
    app::run(ctx, focus)
}
