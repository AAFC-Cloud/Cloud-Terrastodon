use crate::AzureDevOpsBuildFolderPruneSkipReason;
use crate::azure_devops_build_folder_snapshot::AzureDevOpsBuildFolderSnapshot;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;

/// Pruning inventory with unambiguous folder identities and resolved draft paths.
/// Its private storage ensures candidate checks use validated inventory only.
pub(crate) struct AzureDevOpsBuildFolderValidatedSnapshot {
    folders: BTreeMap<String, AzureDevOpsBuildFolderPath>,
    definitions: Vec<AzureDevOpsBuildFolderPath>,
}

impl AzureDevOpsBuildFolderValidatedSnapshot {
    pub(crate) fn new(snapshot: AzureDevOpsBuildFolderSnapshot) -> Result<Self> {
        let mut folders = BTreeMap::<String, AzureDevOpsBuildFolderPath>::new();
        for folder in snapshot.folders {
            let path = folder.path;
            let key = path.case_insensitive_key();
            if let Some(previous) = folders.get(&key) {
                ensure!(
                    previous == &path,
                    "Ambiguous build folder paths: {:?} and {:?}",
                    previous,
                    path
                );
            } else {
                folders.insert(key, path);
            }
        }
        let mut definitions = snapshot
            .definitions
            .iter()
            .map(|definition| definition.path.clone())
            .collect::<Vec<_>>();
        // The API can return drafts as references on another definition. Their
        // folders are occupied even when the draft isn't a top-level row.
        for definition in &snapshot.definitions {
            let references = definition
                .drafts
                .iter()
                .flatten()
                .chain(definition.draft_of.iter());
            for reference in references {
                let path = reference
                    .path
                    .as_ref()
                    .or_else(|| {
                        snapshot
                            .definitions
                            .iter()
                            .find(|definition| definition.id == reference.id)
                            .map(|definition| &definition.path)
                    })
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "Folder path is missing for referenced build definition {}",
                            reference.id
                        )
                    })?;
                definitions.push(path.clone());
            }
        }
        Ok(Self {
            folders,
            definitions,
        })
    }

    pub(crate) fn candidates(
        &self,
        scope: &AzureDevOpsBuildFolderPath,
    ) -> Vec<AzureDevOpsBuildFolderPath> {
        let mut candidates = self
            .folders
            .values()
            .filter(|path| !path.is_root() && scope.contains(path) && !self.occupied(path))
            .cloned()
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right.depth().cmp(&left.depth()).then_with(|| {
                left.case_insensitive_key()
                    .cmp(&right.case_insensitive_key())
            })
        });
        candidates
    }

    pub(crate) fn skip_reason(
        &self,
        candidate: &AzureDevOpsBuildFolderPath,
    ) -> Option<AzureDevOpsBuildFolderPruneSkipReason> {
        match self.folders.get(&candidate.case_insensitive_key()) {
            None => Some(AzureDevOpsBuildFolderPruneSkipReason::NoLongerExists),
            Some(current) if current != candidate => {
                Some(AzureDevOpsBuildFolderPruneSkipReason::PathChanged)
            }
            Some(_) if self.occupied(candidate) => {
                Some(AzureDevOpsBuildFolderPruneSkipReason::ContainsDefinitions)
            }
            Some(_)
                if self
                    .folders
                    .values()
                    .any(|path| path != candidate && candidate.contains(path)) =>
            {
                Some(AzureDevOpsBuildFolderPruneSkipReason::HasChildFolders)
            }
            Some(_) => None,
        }
    }

    fn occupied(&self, path: &AzureDevOpsBuildFolderPath) -> bool {
        self.definitions
            .iter()
            .any(|definition| path.contains(definition))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionId;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionReference;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionSummary;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;

    fn snapshot(
        folders: &[&str],
        definitions: Vec<AzureDevOpsBuildDefinitionReference>,
    ) -> AzureDevOpsBuildFolderSnapshot {
        AzureDevOpsBuildFolderSnapshot {
            folders: folders
                .iter()
                .map(|path| AzureDevOpsBuildFolder {
                    path: path.parse().unwrap(),
                    description: None,
                    project: None,
                    created_by: None,
                    created_on: None,
                    last_changed_by: None,
                    last_changed_date: None,
                })
                .collect(),
            definitions,
        }
    }

    fn definition(id: i32, path: &str) -> AzureDevOpsBuildDefinitionReference {
        AzureDevOpsBuildDefinitionReference {
            id: AzureDevOpsBuildDefinitionId::new(id).unwrap(),
            name: "Synthetic pipeline".parse().unwrap(),
            path: path.parse().unwrap(),
            revision: None,
            quality: None,
            drafts: None,
            draft_of: None,
            queue_status: None,
            r#type: None,
            url: None,
            project: None,
            links: None,
        }
    }

    fn draft(id: i32, path: Option<&str>) -> AzureDevOpsBuildDefinitionSummary {
        AzureDevOpsBuildDefinitionSummary {
            id: AzureDevOpsBuildDefinitionId::new(id).unwrap(),
            name: None,
            path: path.map(|path| path.parse().unwrap()),
            revision: None,
            queue_status: None,
            r#type: None,
            url: None,
            project: None,
        }
    }

    #[test]
    fn candidates_preserve_root_and_occupied_ancestors_and_order_children_first() {
        let inventory = AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(
            &[r"\", r"\Empty", r"\Empty\Child", r"\Used", r"\Used\Child"],
            vec![definition(1, r"\Used\Child")],
        ))
        .unwrap();
        let candidates = inventory.candidates(&AzureDevOpsBuildFolderPath::root());
        assert_eq!(
            candidates
                .iter()
                .map(|path| path.as_str())
                .collect::<Vec<_>>(),
            [r"\Empty\Child", r"\Empty"]
        );
    }

    #[test]
    fn candidates_respect_scope_segments_and_case_without_hiding_occupancy() {
        let inventory = AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(
            &[r"\Foo", r"\Foo\Empty", r"\Foobar", r"\Other"],
            vec![definition(1, r"\fOO\Occupied")],
        ))
        .unwrap();
        let candidates = inventory.candidates(&r"\foo".parse().unwrap());
        assert_eq!(
            candidates
                .iter()
                .map(|path| path.as_str())
                .collect::<Vec<_>>(),
            [r"\Foo\Empty"]
        );
    }

    #[test]
    fn fresh_inventory_protects_changed_missing_or_occupied_candidates() {
        use AzureDevOpsBuildFolderPruneSkipReason::*;
        let candidate = r"\Empty".parse().unwrap();
        for (folders, definitions, expected) in [
            (vec![r"\Empty"], vec![], None),
            (vec![], vec![], Some(NoLongerExists)),
            (vec![r"\EMPTY"], vec![], Some(PathChanged)),
            (
                vec![r"\Empty"],
                vec![definition(1, r"\empty\NewPipeline")],
                Some(ContainsDefinitions),
            ),
            (
                vec![r"\Empty", r"\Empty\NewChild"],
                vec![],
                Some(HasChildFolders),
            ),
        ] {
            let inventory =
                AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(&folders, definitions))
                    .unwrap();
            assert_eq!(inventory.skip_reason(&candidate), expected);
        }
    }

    #[test]
    fn nested_draft_references_protect_their_folder_and_ancestors() {
        for is_draft_of in [false, true] {
            let mut parent = definition(1, r"\Other");
            let reference = draft(2, Some(r"\Drafts\Child"));
            if is_draft_of {
                parent.draft_of = Some(reference);
            } else {
                parent.drafts = Some(vec![reference]);
            }
            let inventory = AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(
                &[r"\Drafts", r"\Drafts\Child"],
                vec![parent],
            ))
            .unwrap();
            assert!(
                inventory
                    .candidates(&AzureDevOpsBuildFolderPath::root())
                    .is_empty()
            );
            for path in [r"\Drafts", r"\Drafts\Child"] {
                assert_eq!(
                    inventory.skip_reason(&path.parse().unwrap()),
                    Some(AzureDevOpsBuildFolderPruneSkipReason::ContainsDefinitions)
                );
            }
        }
    }

    #[test]
    fn inventory_rejects_ambiguous_spelling_and_unresolved_draft_identity() {
        assert!(
            AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(&[r"\Foo", r"\foo"], vec![],))
                .is_err()
        );

        let mut parent = definition(1, r"\Other");
        parent.drafts = Some(vec![draft(2, None)]);
        assert!(
            AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(
                &[r"\Drafts"],
                vec![parent.clone()],
            ))
            .is_err()
        );
        let resolved = AzureDevOpsBuildFolderValidatedSnapshot::new(snapshot(
            &[r"\Drafts"],
            vec![parent, definition(2, r"\Drafts\Child")],
        ))
        .unwrap();
        assert!(
            resolved
                .candidates(&AzureDevOpsBuildFolderPath::root())
                .is_empty()
        );
    }
}
