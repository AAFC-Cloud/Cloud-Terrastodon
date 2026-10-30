use crate::ExcalidrawInteractiveDocument;
use crate::ExcalidrawScene;
use crate::interactive_scene_fields::deleted;
use crate::interactive_scene_fields::edit;
use crate::interactive_scene_fields::set;
use crate::interactive_scene_fields::string;
use eyre::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

impl ExcalidrawScene {
    /// Replace the displayed drawing, including user edits, while retaining the
    /// revision and deletion evidence needed by an open editor's scene merge.
    /// This is deliberately a disposable rendering boundary, not an annotation
    /// merge. Call it under the shared file lock with the latest disk snapshot.
    pub fn replacement_json(&self, previous: Option<&str>) -> Result<String> {
        let mut next = ExcalidrawInteractiveDocument::from_json(&self.to_json()?)?;
        let Some(previous) = previous else {
            return next.to_json();
        };
        let old = ExcalidrawInteractiveDocument::from_json(previous)?;
        let by_id: BTreeMap<_, _> = old
            .elements()
            .iter()
            .filter_map(|element| string(element, "id").map(|id| (id, element)))
            .collect();
        let mut retained = BTreeSet::new();
        for element in next.elements_mut() {
            let Some(id) = string(element, "id") else {
                continue;
            };
            retained.insert(id.clone());
            if let Some(previous) = by_id.get(&id) {
                // Starting from the old revision makes the replacement win over
                // previously saved canvas edits, even when generated IDs recur.
                let replacement = element.clone();
                *element = (*previous).clone();
                edit(element, |current| {
                    let version = current.get("version").cloned();
                    *current = replacement;
                    if let Some(version) = version {
                        current.insert("version".into(), version);
                    }
                    Ok(())
                })?;
            }
        }
        for previous in old.elements() {
            if string(previous, "id").is_some_and(|id| retained.contains(&id)) {
                continue;
            }
            let mut tombstone = previous.clone();
            if !deleted(&tombstone) {
                edit(&mut tombstone, |element| set(element, "isDeleted", &true))?;
            }
            next.elements_mut().push(tombstone);
        }
        next.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExcalidrawObjectSnapshot;
    use crate::interactive_scene_fields::number;

    #[test]
    fn replacing_a_view_overrides_saved_geometry_and_tombstones_removed_objects() -> Result<()> {
        let objects: Vec<_> = ["first", "second"]
            .into_iter()
            .map(|address| ExcalidrawObjectSnapshot {
                address: address.into(),
                shape: "Example".into(),
                fields: vec![],
                fields_complete: true,
                plan: None,
            })
            .collect();
        let first = ExcalidrawScene::from_objects("First view", &objects)?;
        let mut previous = ExcalidrawInteractiveDocument::from_json(&first.to_json()?)?;
        let changed_id = string(&previous.elements()[0], "id").unwrap();
        set(&mut previous.elements_mut()[0], "version", &42)?;
        set(&mut previous.elements_mut()[0], "x", &9000.0)?;

        let scene = ExcalidrawScene::from_objects("Second view", &objects[..1])?;
        let expected = ExcalidrawInteractiveDocument::from_json(&scene.to_json()?)?;
        let replacement = ExcalidrawInteractiveDocument::from_json(
            &scene.replacement_json(Some(&previous.to_json()?))?,
        )?;
        let replaced = replacement
            .elements()
            .iter()
            .find(|element| string(element, "id").as_ref() == Some(&changed_id))
            .unwrap();
        assert!(number(replaced, "version").unwrap() > 42.0);
        assert_eq!(number(replaced, "x"), number(&expected.elements()[0], "x"));
        let expected_ids: BTreeSet<_> = expected
            .elements()
            .iter()
            .filter_map(|element| string(element, "id"))
            .collect();
        let obsolete: Vec<_> = replacement
            .elements()
            .iter()
            .filter(|element| !expected_ids.contains(&string(element, "id").unwrap()))
            .collect();
        assert!(!obsolete.is_empty());
        assert!(obsolete.into_iter().all(deleted));
        assert_eq!(
            replacement
                .elements()
                .iter()
                .filter(|element| !deleted(element))
                .count(),
            expected.elements().len()
        );
        let repeat = ExcalidrawInteractiveDocument::from_json(
            &scene.replacement_json(Some(&replacement.to_json()?))?,
        )?;
        assert_eq!(repeat.elements().len(), replacement.elements().len());
        Ok(())
    }
}
