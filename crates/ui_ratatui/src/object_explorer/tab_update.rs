use super::breadcrumb::Breadcrumb;
use super::breadcrumbs::Breadcrumbs;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TabUpdate {
    Rename(String),
    PushBreadcrumb(Breadcrumb),
    ReplaceBreadcrumb {
        index: usize,
        breadcrumb: Breadcrumb,
    },
    RemoveBreadcrumb(usize),
    ReplaceBreadcrumbs(Breadcrumbs),
}
