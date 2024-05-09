pub trait AsTofuString {
    fn as_tofu_string(&self) -> String;
}

pub trait Sanitizable {
    fn sanitize(&self) -> String;
}

impl<T: AsRef<str>> Sanitizable for T {
    fn sanitize(&self) -> String {
        self.as_ref()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .skip_while(|c| *c == '_')
            .collect()
    }
}
