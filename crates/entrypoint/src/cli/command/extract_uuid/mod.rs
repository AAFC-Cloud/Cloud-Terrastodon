use clap::Args;
use eyre::Result;
use std::io::Read;
use std::io::Write;
use uuid::Uuid;

/// Extract UUIDs from text input
#[derive(Args, Debug, Clone)]
pub struct ExtractUuidArgs {
    /// Input string or '-' to read from stdin
    pub input: String,
}

impl ExtractUuidArgs {
    pub async fn invoke(self) -> Result<()> {
        // read input
        let data = if self.input == "-" {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            s
        } else {
            self.input
        };

        let results = extract_uuids(&data);

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for uuid in results {
            writeln!(out, "{}", uuid)?;
        }

        Ok(())
    }
}

/// Return unique UUIDs found in `data` in order of first appearance
pub fn extract_uuids(data: &str) -> Vec<String> {
    let bytes = data.as_bytes();
    let mut seen = std::collections::HashSet::new();
    let mut results = Vec::new();

    // Try both hyphenated (36) and non-hyphenated (32) forms
    for start in 0..bytes.len() {
        for &len in &[36usize, 32usize] {
            if start + len > bytes.len() {
                continue;
            }
            let slice = &bytes[start..start + len];
            // skip slices that contain nul or control characters
            if slice.iter().any(|b| *b == 0 || (*b as char).is_control()) {
                continue;
            }
            if let Ok(s) = std::str::from_utf8(slice)
                && let Ok(u) = Uuid::parse_str(s)
            {
                let s = u.to_string();
                if seen.insert(s.clone()) {
                    results.push(s);
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::extract_uuids;

    #[test]
    fn find_hyphenated_and_non_hyphenated() {
        let input = "Some text 550e8400-e29b-41d4-a716-446655440000 and 550e8400e29b41d4a716446655440001 somewhere";
        let found = extract_uuids(input);
        assert_eq!(found.len(), 2);
        assert!(found[0].starts_with("550e8400-e29b-41d4-a716-446655440000"));
        assert!(found[1].starts_with("550e8400-e29b-41d4-a716-446655440001"));
    }

    #[test]
    fn deduplicates_preserving_order() {
        let input = "x 550e8400-e29b-41d4-a716-446655440000 y 550e8400-e29b-41d4-a716-446655440000";
        let found = extract_uuids(input);
        assert_eq!(found.len(), 1);
    }
}
