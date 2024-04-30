//! Utility functions.

/// Get the top left coordinate of a rectangle centered in another rectangle. If the
/// inner rectangle is too large in one dimension, default to 1.
pub fn get_centered_top_left(outer_dim: (u16, u16), inner_dim: (u16, u16)) -> (u16, u16) {
    (
        if inner_dim.0 < outer_dim.0 {
            1 + (outer_dim.0 - inner_dim.0) / 2
        } else {
            1
        },
        if inner_dim.1 < outer_dim.1 {
            1 + (outer_dim.1 - inner_dim.1) / 2
        } else {
            1
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_centered_top_left() {
        assert_eq!(get_centered_top_left((10, 10), (8, 8)), (2, 2));
        assert_eq!(get_centered_top_left((5, 5), (8, 8)), (1, 1));
    }
}
