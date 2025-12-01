use bstr::ByteSlice;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::RetryBehaviour;
use eyre::Context;
use eyre::Result;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tokio::time::Instant;
use tokio::time::sleep_until;

#[tokio::test]
async fn it_works() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["--version"])
        .run_raw()
        .await?;
    println!("{}", result);
    Ok(())
}

#[tokio::test]
async fn it_works_cached() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["--version"])
        .use_cache_dir("version")
        .run_raw()
        .await?;
    println!("{}", result);
    Ok(())
}

#[tokio::test]
async fn it_works_azure() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["graph", "query", "--graph-query"])
        .use_retry_behaviour(RetryBehaviour::Fail)
        .azure_file_arg(
            "query.kql",
            r#"
resourcecontainers
| summarize count()
"#
            .to_string(),
        )
        .run_raw()
        .await?;
    println!("{}", result);
    Ok(())
}

#[tokio::test]
async fn it_works_azure_cached() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["graph", "query", "--graph-query"])
        .use_retry_behaviour(RetryBehaviour::Fail)
        .azure_file_arg(
            "query.kql",
            r#"
resourcecontainers
| summarize count()
"#
            .to_string(),
        )
        .use_cache_dir(PathBuf::from_iter([
            "az",
            "graph",
            "query",
            "count-resource-containers",
        ]))
        .run_raw()
        .await?;
    println!("{}", result);
    Ok(())
}

#[tokio::test]
async fn it_works_azure_cached_valid_for() -> Result<()> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.use_retry_behaviour(RetryBehaviour::Fail);
    cmd.args(["graph", "query", "--graph-query"]);
    cmd.azure_file_arg(
        "query.kql",
        r#"
Resources	
| limit 1
| project CurrentTime = now()
"#
        .to_string(),
    );
    let period = Duration::from_secs(5);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "resource_graph", "current-time"]),
        valid_for: period,
    });

    // we don't want anything between our `await` calls that could mess with the timing
    thread::Builder::new().spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                // fetch and cache
                let t1 = Instant::now();
                let result1 = cmd.run_raw().await?;

                // ensure there is at least 1 second remaining before cache expiry
                let t2 = Instant::now();
                assert!(t2 + Duration::from_secs(1) < t1 + period);

                // fetch using cache
                let result2 = cmd.run_raw().await?;

                // sleep until cache expired
                sleep_until(t2 + period + Duration::from_secs(1)).await;
                let t3 = Instant::now();
                assert!(t3 > t2 + period);

                // fetch new results without using cache
                let result3 = cmd.run_raw().await?;

                // ensure first two match and don't match third
                println!("result1: {result1:?}\nresult2: {result2:?}\nresult3: {result3:?}");
                assert_eq!(result1, result2);
                assert_ne!(result1, result3);
                Ok::<(), eyre::Error>(())
            })
            .unwrap();
    })?;
    Ok(())
}

#[tokio::test]
async fn user() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["ad", "signed-in-user", "show"])
        .use_retry_behaviour(RetryBehaviour::Fail)
        .run_raw()
        .await;
    println!("{:?}", result);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn login() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["login"])
        .run_raw()
        .await?;
    println!("{}", result);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn logout() -> Result<()> {
    let result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["logout"])
        .run_raw()
        .await;
    println!("{:?}", result);
    Ok(())
}
#[tokio::test]
#[ignore]
async fn reauth() -> Result<()> {
    println!("Logging out...");
    let logout_result = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["logout"])
        .run_raw()
        .await;
    match logout_result {
        Ok(msg) => println!("{}", msg),
        Err(e) => match e.downcast_ref::<CommandOutput>() {
            Some(CommandOutput { stderr, .. })
                if stderr.contains_str("ERROR: There are no active accounts.") =>
            {
                println!("Already logged out!")
            }
            _ => {
                return Err(e).context("unknown logout failure");
            }
        },
    }
    println!("Performing command, it should prompt for login...");
    println!(
        "{}",
        CommandBuilder::new(CommandKind::AzureCLI)
            .args(["ad", "signed-in-user", "show"])
            .run_raw()
            .await?
    );
    Ok(())
}
