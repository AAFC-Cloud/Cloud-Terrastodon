use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use http::Method;

pub fn build_arm_rest_command(method: Method, url: &str, cache_key: CacheKey) -> CommandBuilder {
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    let method = method.as_str();
    cmd.args(["rest", "--method", method, "--url", url]);
    cmd.cache(cache_key);
    cmd
}

pub fn build_arm_rest_get_command(url: &str, cache_key: CacheKey) -> CommandBuilder {
    build_arm_rest_command(Method::GET, url, cache_key)
}
