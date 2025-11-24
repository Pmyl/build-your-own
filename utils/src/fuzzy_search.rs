pub fn fuzzy_search(search: &str, strings: &[&str], threshold: usize) -> Vec<usize> {
    let mut matches = vec![];
    for (i, string) in strings.iter().enumerate() {
        if distance(string, search) <= threshold {
            matches.push(i);
        }
    }
    matches.sort();
    matches
}

fn distance(a: &str, b: &str) -> usize {
    let mut row_prev_2 = Vec::<usize>::with_capacity(b.len() + 1);
    let mut row_prev_1 = (0..b.len() + 1).collect::<Vec<usize>>();
    let mut row = Vec::<usize>::with_capacity(b.len() + 1);
    let a_chars = a.chars().collect::<Vec<_>>();
    let b_chars = b.chars().collect::<Vec<_>>();

    // Loop rows one by one
    for row_i in 1..=a_chars.len() {
        // First column is not pre-calculated because it's easy to create in the loop
        row.push(row_i);

        // Loop colums one by one (i.e. each element of the current row)
        for col_i in 1..=b_chars.len() {
            let a_char = a_chars[row_i - 1];
            let b_char = b_chars[col_i - 1];

            let substitution_cost = if a_char == b_char { 0 } else { 1 };
            let mut distance = (row_prev_1[col_i] + 1) // deletion
                .min(row[col_i - 1] + 1) // insertion
                .min(row_prev_1[col_i - 1] + substitution_cost); // substitution

            // transposition
            if row_i > 1
                && col_i > 1
                && a_chars[row_i - 2] == b_char
                && b_chars[col_i - 2] == a_char
            {
                distance = distance.min(row_prev_2[col_i - 2] + 1);
            }

            row.push(distance);
        }

        std::mem::swap(&mut row_prev_2, &mut row_prev_1);
        std::mem::swap(&mut row_prev_1, &mut row);
        row.clear();
    }

    row_prev_1.last().copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_lavenshtein_distance() {
        assert_eq!(distance("", ""), 0);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("rust", "rust"), 0);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("flaw", "lawn"), 2);
        // Unicode test
        assert_eq!(distance("é", "e"), 1);
    }

    #[test]
    fn damerau_transposition_tests() {
        // Simple adjacent swaps
        assert_eq!(distance("rsut", "rust"), 1); // swap s-u
        assert_eq!(distance("srut", "rust"), 2); // two swaps needed
        assert_eq!(distance("acbd", "abcd"), 1); // swap c-b

        // No swaps, just regular edits
        assert_eq!(distance("abc", "abc"), 0);
        assert_eq!(distance("abc", "bac"), 1); // single adjacent swap

        // Multiple swaps
        assert_eq!(distance("ca", "ac"), 1); // single swap
        assert_eq!(distance("dcba", "cdab"), 2); // two swaps needed

        // Mix of swaps and substitutions
        assert_eq!(distance("abdc", "abcd"), 1); // swap d-c

        // Edge case: empty strings
        assert_eq!(distance("", ""), 0);
        assert_eq!(distance("", "a"), 1);
        assert_eq!(distance("a", ""), 1);
    }

    #[test]
    fn damerau_transposition_edge_cases() {
        assert_eq!(distance("abab", "baba"), 2);
        assert_eq!(distance("ruts", "rusto"), 2);
        assert_eq!(distance("coffee", "cafe"), 3);
    }

    #[test]
    fn fuzzy_search_basic_tests() {
        let words = ["rust", "python", "go", "ruby"];

        // Exact match
        assert_eq!(fuzzy_search("rust", &words, 0), vec![0]);

        // Small typo
        assert_eq!(fuzzy_search("rsut", &words, 1), vec![0]);

        // Threshold too low -> no match
        assert_eq!(fuzzy_search("rsut", &words, 0), vec![]);

        // Multiple close matches
        let words2 = ["rust", "rusto", "rusty"];
        assert_eq!(fuzzy_search("ruts", &words2, 1), vec![0]); // rust=0, rusto=2, rusty=2

        // Threshold includes all matches up to 2
        assert_eq!(fuzzy_search("ruts", &words2, 2), vec![0, 1, 2]); // rusty distance=2
    }

    #[test]
    fn fuzzy_search_unicode_tests() {
        let words = ["café", "coffee", "caffè"];

        // Small typo
        assert_eq!(fuzzy_search("cafe", &words, 1), vec![0]);

        // Exact match
        assert_eq!(fuzzy_search("coffee", &words, 0), vec![1]);
    }
}
