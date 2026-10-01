use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

use crate::limits::MAX_VEC_LEN;

/// Deserializes a `Vec<T>` while enforcing `MAX_VEC_LEN`.
///
/// Rejects inputs with more than 10,000 elements to prevent
/// resource exhaustion from adversarially large arrays.
pub(crate) fn deserialize_bounded_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(BoundedVecVisitor(PhantomData))
}

struct BoundedVecVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for BoundedVecVisitor<T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a sequence with at most {} elements",
            MAX_VEC_LEN
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Use the size hint but cap it so a lying hint can't OOM us.
        let capacity = seq.size_hint().unwrap_or(0).min(MAX_VEC_LEN);
        let mut vec = Vec::with_capacity(capacity);

        while let Some(elem) = seq.next_element()? {
            if vec.len() >= MAX_VEC_LEN {
                return Err(de::Error::custom(format!(
                    "sequence length exceeds maximum of {}",
                    MAX_VEC_LEN
                )));
            }
            vec.push(elem);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: deserialize a RON sequence of u32 through the bounded visitor.
    fn deser_bounded(input: &str) -> Result<Vec<u32>, String> {
        let mut deser = ron::Deserializer::from_str(input).unwrap();
        deserialize_bounded_vec::<u32, _>(&mut deser).map_err(|e| e.to_string())
    }

    #[test]
    fn accepts_empty_vec() {
        let result = deser_bounded("[]").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn accepts_small_vec() {
        let result = deser_bounded("[1, 2, 3]").unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn accepts_exactly_max_vec_len() {
        // Build a RON array with exactly MAX_VEC_LEN elements
        let mut input = String::from("[");
        for i in 0..MAX_VEC_LEN {
            if i > 0 {
                input.push_str(", ");
            }
            input.push_str(&(i as u32).to_string());
        }
        input.push(']');

        let result = deser_bounded(&input).unwrap();
        assert_eq!(result.len(), MAX_VEC_LEN);
    }

    #[test]
    fn rejects_vec_exceeding_max_len() {
        // Build a RON array with MAX_VEC_LEN + 1 elements
        let mut input = String::from("[");
        for i in 0..=MAX_VEC_LEN {
            if i > 0 {
                input.push_str(", ");
            }
            input.push('0');
        }
        input.push(']');

        let result = deser_bounded(&input);
        assert!(
            result.is_err(),
            "should reject vec with {} elements",
            MAX_VEC_LEN + 1
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("exceeds maximum"),
            "error should mention exceeding maximum, got: {}",
            err_msg
        );
    }
}
