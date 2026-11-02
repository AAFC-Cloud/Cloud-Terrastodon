use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use std::collections::HashSet;

/// Converts a value to HCL text.
pub trait AsHclString {
    fn as_hcl_string(&self) -> String;
}

impl AsHclString for String {
    fn as_hcl_string(&self) -> String {
        self.to_owned()
    }
}

impl AsHclString for &str {
    fn as_hcl_string(&self) -> String {
        self.to_string()
    }
}

impl<T> AsHclString for Vec<T>
where
    T: AsHclString,
{
    fn as_hcl_string(&self) -> String {
        let mut rtn = String::new();
        for v in self.iter() {
            rtn.push_str(v.as_hcl_string().as_str());
            rtn.push('\n');
        }
        rtn
    }
}

impl<T> AsHclString for HashSet<T>
where
    T: AsHclString,
{
    fn as_hcl_string(&self) -> String {
        let mut rtn = String::new();
        for v in self.iter() {
            rtn.push_str(v.as_hcl_string().as_str());
            rtn.push('\n');
        }
        rtn
    }
}

impl AsHclString for Block {
    fn as_hcl_string(&self) -> String {
        Body::builder().block(self.clone()).build().to_string()
    }
}

impl AsHclString for Body {
    fn as_hcl_string(&self) -> String {
        self.to_string()
    }
}
