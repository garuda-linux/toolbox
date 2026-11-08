//! "Did you mean": the known name closest to one the system lacks.

#[cfg(test)]
mod tests;

/// Units are often typed without their suffix.
const SERVICE: &str = ".service";

const MAX_EDITS: usize = 3;

/// The name among `candidates` closest to `wanted`, ignoring case and a
/// missing `.service`, when it is close enough to be a typo.
pub fn suggest<'a, I>(wanted: &str, candidates: I) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    let wanted = wanted.to_lowercase();
    let length = wanted.chars().count();
    let allowed = (length / 4).clamp(1, MAX_EDITS);

    candidates
        .into_iter()
        .map(|candidate| (edits(&wanted, candidate), candidate))
        .filter(|(edits, _)| *edits <= allowed && *edits < length)
        .min_by_key(|(edits, _)| *edits)
        .map(|(_, candidate)| candidate)
}

pub fn did_you_mean<'a, I>(wanted: &str, candidates: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    suggest(wanted, candidates).map(|found| format!("did you mean `{found}`?"))
}

/// The edits from `wanted` to `candidate`, or to it without `.service`
/// when `wanted` has no suffix of its own.
fn edits(wanted: &str, candidate: &str) -> usize {
    let candidate = candidate.to_lowercase();
    let full = distance(wanted, &candidate);

    match candidate.strip_suffix(SERVICE) {
        Some(stem) if !wanted.ends_with(SERVICE) => full.min(distance(wanted, stem)),
        _ => full,
    }
}

/// The optimal string alignment distance over characters: Levenshtein's,
/// with two swapped neighbours as one edit, the commonest typo.
fn distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut before: Vec<usize> = Vec::new();
    let mut previous: Vec<usize> = (0..=right.len()).collect();

    for (row, left_char) in left.iter().enumerate() {
        let mut current = vec![row + 1];

        for (column, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            let mut best = (at(&previous, column) + cost)
                .min(at(&previous, column + 1) + 1)
                .min(at(&current, column) + 1);

            let swapped = row > 0
                && column > 0
                && left.get(row - 1) == Some(right_char)
                && right.get(column - 1) == Some(left_char);

            if swapped {
                best = best.min(at(&before, column - 1) + 1);
            }

            current.push(best);
        }

        before = std::mem::replace(&mut previous, current);
    }

    at(&previous, right.len())
}

/// A cell of a row, far off for one outside it.
fn at(row: &[usize], column: usize) -> usize {
    row.get(column).copied().unwrap_or(usize::MAX / 2)
}
