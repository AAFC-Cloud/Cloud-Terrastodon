use cloud_terrastodon_relative_location::RelativeLocation;

/**
 * This example doesn't actually demonstrate what this crate is trying to solve since it only manifests when
 * crossing crate boundaries calling a #[track_caller] fn.
 */
#[track_caller]
pub fn do_work() -> eyre::Result<()> {
    match work_inner() {
        Ok(x) => Ok(x),
        Err(e) => Err(eyre::Error::msg(e).wrap_err(format!(
            "Called from {}",
            RelativeLocation::from(std::panic::Location::caller())
        ))),
    }
}

#[track_caller]
pub fn do_work_outer() -> eyre::Result<()> {
    do_work()
}

pub fn work_inner() -> Result<(), String> {
    Err("Oh no! This is intentionally failing".to_string())
}

pub fn main() -> eyre::Result<()> {
    do_work_outer()?;
    Ok(())
}
