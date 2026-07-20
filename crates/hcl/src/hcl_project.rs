use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;

/// The parsed HCL files that make up a Terraform project.
#[derive(Debug, Default)]
pub struct HclProject(HashMap<PathBuf, Body>);

impl HclProject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> HashMap<PathBuf, Body> {
        self.0
    }

    pub fn as_map(&self) -> &HashMap<PathBuf, Body> {
        &self.0
    }

    pub fn as_mut_map(&mut self) -> &mut HashMap<PathBuf, Body> {
        &mut self.0
    }

    pub fn into_values(self) -> std::collections::hash_map::IntoValues<PathBuf, Body> {
        self.0.into_values()
    }
}

impl From<HashMap<PathBuf, Body>> for HclProject {
    fn from(value: HashMap<PathBuf, Body>) -> Self {
        Self(value)
    }
}

impl From<HclProject> for HashMap<PathBuf, Body> {
    fn from(value: HclProject) -> Self {
        value.0
    }
}

impl<const N: usize> From<[(PathBuf, Body); N]> for HclProject {
    fn from(value: [(PathBuf, Body); N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(PathBuf, Body)> for HclProject {
    fn from_iter<T: IntoIterator<Item = (PathBuf, Body)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for HclProject {
    type Item = (PathBuf, Body);
    type IntoIter = std::collections::hash_map::IntoIter<PathBuf, Body>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a HclProject {
    type Item = (&'a PathBuf, &'a Body);
    type IntoIter = std::collections::hash_map::Iter<'a, PathBuf, Body>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut HclProject {
    type Item = (&'a PathBuf, &'a mut Body);
    type IntoIter = std::collections::hash_map::IterMut<'a, PathBuf, Body>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Deref for HclProject {
    type Target = HashMap<PathBuf, Body>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HclProject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
