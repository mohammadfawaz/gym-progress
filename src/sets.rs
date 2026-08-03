//! Set-rep formatting lives here so it can be tested independently of the UI.

pub(crate) fn defaults() -> [u32; 3] {
    [10, 10, 10]
}

pub(crate) fn parse(reps: &str) -> [u32; 3] {
    let mut slots = defaults();
    for (index, part) in reps.split(',').take(3).enumerate() {
        if let Ok(value) = part.trim().parse::<u32>() {
            slots[index] = value;
        }
    }
    slots
}

pub(crate) fn format(slots: &[u32; 3]) -> String {
    slots
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_partial_or_invalid_set_input_without_losing_defaults() {
        assert_eq!(parse("8, 7, 6"), [8, 7, 6]);
        assert_eq!(parse("8, nope"), [8, 10, 10]);
    }
}
