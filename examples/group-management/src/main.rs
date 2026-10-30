use cloud_terrastodon::app::App;
use cloud_terrastodon::app::AuthSource;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::prepare_process;
use cloud_terrastodon_group_management::cli::GroupManagementArgs;
use cloud_terrastodon_group_management::cli::GroupManagementCommand;
use std::process::ExitCode;
use std::process::Termination;

fn main() -> Result<()> {
    prepare_process()?;
    let app = App::new("group-management", env!("CARGO_PKG_VERSION"));
    let mut cli = match app
        .parse_from::<GroupManagementArgs>(std::env::args_os().skip(1))
        .into_result()
    {
        Ok(output) => output.get(),
        Err(outcome) => {
            let status = outcome.report();
            std::process::exit(if status == ExitCode::SUCCESS { 0 } else { 1 });
        }
    };
    if matches!(&cli.command, GroupManagementCommand::Validate(_)) {
        cli.global_args.auth_source = AuthSource::None;
    }
    app.run_parsed(cli, |cli, context| async move {
        let output_format = cli.global_args.output_format;
        let output = cli.invoke(&context.auth, &context.cancellation).await?;
        output.emit(output_format, &context.cancellation)
    })
}
