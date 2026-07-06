//! Valence-electron derivation from an electron-configuration string.

/// Group-based valence fallback when the configuration cannot be parsed.
/// Main-group elements only; transition metals (groups 3..=12) return 0.
pub fn group_to_valence_fallback(group: u8) -> u8 {
    if group <= 2 {
        group
    } else if group >= 13 {
        group - 10
    } else {
        0
    }
}

/// Whether `group` is a d-block transition metal (groups 3..=12).
pub fn is_transition_metal(group: u8) -> bool {
    (3..=12).contains(&group)
}

/// Parse the count of electrons in the highest principal shell from a
/// configuration like `"1s2 2s2 2p6 3s1"` or `"[Ne] 3s2 3p3"`.
///
/// Falls back to [`group_to_valence_fallback`] when the string is empty (after
/// stripping a noble-gas prefix) or has no parseable subshell tokens.
pub fn parse_valence_electrons(config: &str, group: u8) -> u8 {
    let stripped = strip_noble_gas_prefix(config);
    if stripped.is_empty() {
        return group_to_valence_fallback(group);
    }

    // (principal quantum number, electron count) for each parseable subshell.
    let mut subshells: Vec<(u8, u8)> = Vec::new();
    for token in stripped.split_whitespace() {
        if let Some(parsed) = parse_subshell(token) {
            subshells.push(parsed);
        }
    }
    if subshells.is_empty() {
        return group_to_valence_fallback(group);
    }

    let max_n = subshells.iter().map(|s| s.0).max().expect("non-empty");
    subshells.iter().filter(|s| s.0 == max_n).map(|s| s.1).sum()
}

/// Drop a leading noble-gas prefix such as `"[Ne] "` and trim surrounding space.
fn strip_noble_gas_prefix(config: &str) -> &str {
    let trimmed = config.trim();
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(close) = rest.find(']') {
            return rest[close + 1..].trim();
        }
    }
    trimmed
}

/// Parse a single `"<n><spdf><count>"` token, e.g. `"3p5"` -> `(3, 5)`.
fn parse_subshell(token: &str) -> Option<(u8, u8)> {
    let mut chars = token.chars();
    let n = chars.next()?.to_digit(10)? as u8;
    let subshell = chars.next()?;
    if !matches!(subshell, 's' | 'p' | 'd' | 'f') {
        return None;
    }
    let count: String = chars.collect();
    if count.is_empty() {
        return None;
    }
    let count = count.parse::<u8>().ok()?;
    Some((n, count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback() {
        assert_eq!(group_to_valence_fallback(1), 1);
        assert_eq!(group_to_valence_fallback(2), 2);
        assert_eq!(group_to_valence_fallback(14), 4);
        assert_eq!(group_to_valence_fallback(17), 7);
        assert_eq!(group_to_valence_fallback(8), 0); // transition: no fallback
    }

    #[test]
    fn transition_metal_range() {
        assert!(is_transition_metal(3));
        assert!(is_transition_metal(12));
        assert!(!is_transition_metal(2));
        assert!(!is_transition_metal(13));
    }

    #[test]
    fn parse_highest_shell() {
        assert_eq!(parse_valence_electrons("1s2 2s2 2p6 3s1", 1), 1); // Na
        assert_eq!(parse_valence_electrons("1s2 2s2 2p6 3s2 3p5", 17), 7); // Cl
        assert_eq!(parse_valence_electrons("1s2 2s2 2p6 3s2 3p6 3d6 4s2", 8), 2);
        // Fe
    }

    #[test]
    fn strips_noble_gas_prefix() {
        assert_eq!(parse_valence_electrons("[Ne] 3s2 3p3", 15), 5);
    }

    #[test]
    fn empty_falls_back_to_group() {
        assert_eq!(parse_valence_electrons("", 16), 6);
        assert_eq!(parse_valence_electrons("[Ar]", 16), 6); // only a prefix
    }

    #[test]
    fn unparseable_tokens_fall_back() {
        assert_eq!(parse_valence_electrons("xx yy", 13), 3);
    }
}
