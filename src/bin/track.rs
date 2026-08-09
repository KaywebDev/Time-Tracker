// Entry point for the track application
use track::{error::{AppError, Suggestion}, init};
use error_stack::{Report, Result, ResultExt};

fn main() -> Result<(), AppError> {
    init::error_reporting();

    return Err(Report::from(AppError)).attach(Suggestion("Do something else"));

    Ok(())
}