use nucleo::Config;
use nucleo::Matcher;
use nucleo::Utf32Str;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;

/// Use the same fuzzy matching engine and smart case/Unicode normalization as
/// the terminal picker. Equal scores retain the caller's recommended ordering.
pub(super) fn rank<'a>(query: &str, names: &'a [String]) -> Vec<&'a String> {
    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut buffer = Vec::new();
    let mut matches: Vec<_> = names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            pattern
                .score(Utf32Str::new(name, &mut buffer), &mut matcher)
                .map(|score| (score, index, name))
        })
        .collect();
    matches.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    matches.into_iter().map(|(_, _, name)| name).collect()
}
