use embed_resource::CompilationResult;

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows") {
        let _: CompilationResult = embed_resource::compile("icon.rc", embed_resource::NONE);
    }
}
