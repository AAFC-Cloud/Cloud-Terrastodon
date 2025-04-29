use hcl::edit::structure::Block;
use itertools::Itertools;

pub trait TofuBlockSortable: Iterator<Item = Block> {
    fn sort_blocks(self) -> std::vec::IntoIter<Block>
    where
        Self: Sized,
    {
        let mut v = Vec::from_iter(self);
        v.sort_by(|a, b| {
            a.ident.as_str().cmp(b.ident.as_str()).then(
                a.labels
                    .iter()
                    .map(|x| x.as_str())
                    .join("")
                    .cmp(&b.labels.iter().map(|x| x.as_str()).join("")),
            )
        });
        v.into_iter()
    }
}
impl<T> TofuBlockSortable for T where T: Iterator<Item = Block> + ?Sized {}
