use tracing::{info, warn};
// Entry point for the track application
use track::{error::{AppError, Suggestion}, feature::cli, init};
use error_stack::{Report, Result, ResultExt};

// track is the binary name for this application
//
// track start
// track stop
// track report

fn main() -> Result<(), AppError> {
    init::error_reporting();
    init::tracing();

    cli::run()
        .change_context(AppError)
        .attach_printable("failed to run CLI")?;

    Ok(())
}