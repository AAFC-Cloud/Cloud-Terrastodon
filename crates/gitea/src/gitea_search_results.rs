use facet::Facet;

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct GiteaSearchResults<T> {
    #[facet(default)]
    pub data: Vec<T>,
    #[facet(default)]
    pub ok: bool,
}
