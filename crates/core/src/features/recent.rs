#[cfg(test)]
mod tests;

pub const KEEP: usize = 3;

pub fn remember<T>(recent: &mut Vec<T>, visited: T)
where
    T: PartialEq,
{
    recent.retain(|kept| *kept != visited);
    recent.insert(0, visited);
    recent.truncate(KEEP);
}
