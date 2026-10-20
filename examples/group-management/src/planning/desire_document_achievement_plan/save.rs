use super::DesireDocumentAchievementPlan;
use cloud_terrastodon::app::CancellationScope;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use facet_json::to_writer_std_pretty;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use teamy_cancellation::CancellableWriter;
use teamy_cancellation::CancellationReportExt;

impl DesireDocumentAchievementPlan {
    /// Persist a complete JSON artifact without replacing an existing reviewed plan.
    pub fn save(&self, path: &Path, cancellation: &CancellationScope) -> Result<()> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .wrap_err_with(|| format!("Creating a plan artifact beside {}", path.display()))?;
        {
            let mut buffered = BufWriter::new(&mut temporary);
            let written = (|| {
                let mut writer = CancellableWriter::new(&mut buffered, cancellation);
                to_writer_std_pretty(&mut writer, self)?;
                writer.write_all(b"\n")?;
                writer.flush()
            })();
            // Discard any unfinished buffer without BufWriter's implicit drop-time
            // flush. Only explicit successful flushing can proceed to publication.
            drop(buffered.into_parts());
            written.map_err(|error| Report::new(error).normalize_cancellation())?;
        }
        cancellation.bail_if_cancelled()?;
        temporary.as_file().sync_all()?;
        cancellation.bail_if_cancelled()?;
        temporary
            .persist_noclobber(path)
            .map_err(|error| error.error)
            .wrap_err_with(|| {
                format!(
                    "Saving plan to {} without overwriting an existing artifact",
                    path.display()
                )
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DesireDocumentAchievementPlan;
    use crate::DesireDocument;
    use std::io;
    use teamy_cancellation::CancellationError;
    use teamy_cancellation::CancellationToken;

    #[test]
    fn a_second_save_cannot_replace_a_previously_reviewed_artifact() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let directory = tempfile::tempdir().unwrap();
                let path = directory.path().join("plan.json");
                let desire = DesireDocument::try_new(Vec::new(), "first-document".into()).unwrap();
                let original = DesireDocumentAchievementPlan::try_new(&desire, &[], &[]).unwrap();
                original.save(&path, &cancellation)?;
                let reviewed = std::fs::read(&path).unwrap();
                let desire =
                    DesireDocument::try_new(Vec::new(), "different-document".into()).unwrap();
                let changed = DesireDocumentAchievementPlan::try_new(&desire, &[], &[]).unwrap();
                assert!(changed.save(&path, &cancellation).is_err());
                assert_eq!(std::fs::read(&path).unwrap(), reviewed);
                eyre::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn cancellation_leaves_no_reviewable_artifact_or_temporary_file() {
        let token = CancellationToken::new();
        token
            .with_scope(|cancellation| {
                let directory = tempfile::tempdir().unwrap();
                let path = directory.path().join("plan.json");
                let desire = DesireDocument::try_new(Vec::new(), "document".into()).unwrap();
                let plan = DesireDocumentAchievementPlan::try_new(&desire, &[], &[]).unwrap();
                token.request_cancel("plan saving cancelled");

                let error = plan.save(&path, &cancellation).unwrap_err();

                assert_eq!(
                    error.downcast_ref::<CancellationError>().unwrap().reason,
                    "plan saving cancelled"
                );
                assert!(error.downcast_ref::<io::Error>().is_some());
                assert!(!path.exists());
                assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
                eyre::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn a_protected_save_publishes_a_complete_plan_after_cancellation_is_requested() {
        let token = CancellationToken::new();
        token
            .with_scope(|cancellation| {
                let directory = tempfile::tempdir().unwrap();
                let path = directory.path().join("plan.json");
                let desire =
                    DesireDocument::try_new(Vec::new(), "protected document".into()).unwrap();
                let plan = DesireDocumentAchievementPlan::try_new(&desire, &[], &[]).unwrap();
                let mut expected = facet_json::to_vec_pretty(&plan).unwrap();
                expected.push(b'\n');

                cancellation.with_deferral(
                    "finish the already-required plan artifact",
                    |protected| {
                        token.request_cancel("cancel after the protected save was admitted");
                        plan.save(&path, &protected)
                    },
                )?;

                let bytes = std::fs::read(&path).unwrap();
                assert_eq!(bytes, expected);
                assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
                assert!(cancellation.bail_if_cancelled().is_err());
                eyre::Ok(())
            })
            .unwrap();
    }
}
