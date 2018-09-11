use serde::{
    de::{
        self,
        SeqAccess,
        Visitor,
    },
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt,
    ops::Range,
    sync::{
        atomic::AtomicUsize,
        Arc,
    },
};
use core::{
    str::next_code_point,
    char,
};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Style {
    Normie,   // sentinel
    Selected, // never used except on the client
    Bold,
    Italic,
    Link,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type StyleMap = HashMap<Style, Option<String>>;
pub type StyleSet = HashSet<Style>;

/// Abstraction for String that allows a limited set of operations
/// with good optimization. (Or that's the idea.)
#[derive(Clone, Debug)]
pub struct DocString(Arc<String>, pub Option<Range<usize>>, Option<Arc<StyleMap>>);

impl DocString {
    pub fn from_string(input: String) -> DocString {
        DocString(Arc::new(input), None, None)
    }

    pub fn from_str(input: &str) -> DocString {
        DocString(Arc::new(input.to_owned()), None, None)
    }

    pub fn from_string_styled(input: String, styles: StyleMap) -> DocString {
        DocString(Arc::new(input), None, Some(Arc::new(styles)))
    }

    pub fn from_str_styled(input: &str, styles: StyleMap) -> DocString {
        DocString(Arc::new(input.to_owned()), None, Some(Arc::new(styles)))
    }

    // TODO audit use of this
    pub fn as_str(&self) -> &str {
        if let Some(ref range) = self.1 {
            &self.0[range.clone()]
        } else {
            &self.0
        }
    }

    pub fn styles(&self) -> Option<Arc<StyleMap>> {
        self.2.clone()
    }

    pub fn remove_styles(&mut self, styles: &StyleSet) {
        if let &mut Some(ref mut self_styles) = &mut self.2 {
            let mut new_styles: StyleMap = (**self_styles).clone();
            *self_styles = Arc::new(new_styles
                .drain()
                .filter(|(ref x, _)| !styles.contains(x))
                .collect());
        } else {
            // no-op
        }
    }

    pub fn extend_styles(&mut self, styles: &StyleMap) {
        if let &mut Some(ref self_styles) = &mut self.2 {
            let mut new_styles: StyleMap = (**self_styles).clone();
            new_styles.extend(styles.iter().map(|(a, b)| (a.to_owned(), b.to_owned())));
            self.2 = Some(Arc::new(new_styles));
        } else {
            self.2 = Some(Arc::new(styles.to_owned()));
        }
    }

    // Add text (with the same styling) to the end of this string.
    pub fn push_str(&mut self, input: &str) {
        let mut value = self.to_string();
        value.push_str(input);
        self.0 = Arc::new(value);
        self.1 = None;
    }

    #[inline(never)]
    #[no_mangle]
    pub fn slice_inner(s: &str, b: usize) -> usize {
        let mut iter = s.as_bytes().iter();
        for _ in 0..b {
            next_code_point(&mut iter);
        }
        s.len() - iter.as_slice().len()
    }

    // TODO consume self?
    pub fn split_at(&self, char_boundary: usize) -> (DocString, DocString) {
        let mut start = 0;
        let mut end = self.0.len();
        if let Some(ref range) = self.1 {
            start = range.start;
            end = range.end;
        }

        let byte_index = DocString::slice_inner(&self.0[start..], char_boundary);

        (
            DocString(
                self.0.clone(),
                Some((start + 0)..(start + byte_index)),
                self.2.clone(),
            ),
            DocString(
                self.0.clone(),
                Some((start + byte_index)..end),
                self.2.clone(),
            ),
        )
    }

    pub unsafe fn seek_forward(&mut self, add: usize) {
        let mut start = 0;
        let mut end = self.0.len();
        if let Some(ref range) = self.1 {
            start = range.start;
            end = range.end;
        }
        self.1 = Some(start + add..end); //TODO do not land with this, not utf-8 safe
    }

    pub unsafe fn seek_backward(&mut self, sub: usize) {
        let mut start = 0;
        let mut end = self.0.len();
        if let Some(ref range) = self.1 {
            start = range.start;
            end = range.end;
        }
        self.1 = Some(start - sub..end); //TODO do not land with this, not utf-8 safe
    }

    pub unsafe fn byte_range_mut(&mut self) -> &mut Range<usize> {
        if self.1.is_none() {
            self.1 = Some(0..(self.0.len()));
        }
        self.1.as_mut().unwrap()
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_owned()
    }

    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    pub fn into_string(self) -> String {
        // TODO make this faster by deconstructing the Rc?
        self.to_string()
    }

    pub fn char_len(&self) -> usize {
        self.as_str().chars().count()
    }
}

impl PartialEq for DocString {
    fn eq(&self, other: &DocString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for DocString {}

impl Serialize for DocString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let &Some(ref value) = &self.2 {
            let mut s = serializer.serialize_seq(Some(2))?;
            s.serialize_element(self.as_str())?;
            s.serialize_element(Arc::as_ref(value))?;
            s.end()
        } else {
            serializer.serialize_str(self.as_str())
        }
    }
}

impl<'de> Deserialize<'de> for DocString {
    fn deserialize<D>(deserializer: D) -> Result<DocString, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = DocString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("docstring")
            }

            fn visit_str<E>(self, value: &str) -> Result<DocString, E>
            where
                E: de::Error,
            {
                Ok(DocString::from_str(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<DocString, A::Error>
            where
                A: SeqAccess<'de>,
            {
                if let Some(inner) = seq.next_element::<String>()? {
                    if let Some(styles) = seq.next_element::<StyleMap>()? {
                        Ok(DocString::from_string_styled(inner, styles))
                    } else {
                        Err(de::Error::unknown_field("1", FIELDS))
                    }
                } else {
                    Err(de::Error::unknown_field("0", FIELDS))
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["docstring"];
        deserializer.deserialize_any(FieldVisitor)
    }
}

#[derive(Debug)]
struct DividedString {
    original_range: Range<usize>,
    left_string: DocString,
    right_string: DocString,
    index: usize,
}

impl DividedString {
    fn new(input: DocString, index: usize) -> DividedString {
        let char_len = input.char_len();
        if index > char_len - 1 {
            panic!("Invalid index to DividedString");
        }
        DividedString {
            original_range: input.1.clone().unwrap_or_else(|| (0..char_len)),
            left_string: input.clone(),
            right_string: input,
            index: index,
        }
    }

    fn seek(&mut self, index: isize) {
        if (self.index as isize) + index < 0 {
            panic!("Moved before start of string");
        }
        self.index = ((self.index as isize) + index) as usize;
        if self.index > self.original_range.len() {
            panic!("Moved beyond end of string");
        }
    }

    fn update_left(&mut self) {
        let mut range = self.original_range.clone();
        range.start += self.index;
        self.left_string.1 = Some(range);
    }

    pub fn left<'a>(&'a mut self) -> Option<&'a DocString> {
        if self.index == 0 {
            None
        } else {
            self.update_left();
            Some(&self.left_string)
        }
    }

    fn update_right(&mut self) {
        let mut range = self.original_range.clone();
        range.end = range.start + self.index;
        self.left_string.1 = Some(range);
    }

    pub fn right<'a>(&'a mut self) -> Option<&'a DocString> {
        if self.index >= self.original_range.len() {
            None
        } else {
            self.update_right();
            Some(&self.right_string)
        }
    }

    pub fn destruct(mut self) -> (DocString, DocString) {
        self.update_left();
        self.update_right();
        (self.left_string, self.right_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut ds = DividedString::new(DocString::from_str("Welcome!"), 1);
        ds.seek(5);
        assert_eq!(ds.left().unwrap().as_str(), "e!");
    }

    #[test]
    #[should_panic]
    fn seek_too_far() {
        let mut ds = DividedString::new(DocString::from_str("Welcome!"), 1);
        ds.seek(10);
    }

    #[test]
    #[should_panic]
    fn seek_negative() {
        let mut ds = DividedString::new(DocString::from_str("Welcome!"), 1);
        ds.seek(3);
        ds.seek(-10);
    }

    #[test]
    fn option_ends() {
        let mut ds = DividedString::new(DocString::from_str("Welcome!"), 0);
        assert_eq!(ds.left(), None);
        assert_eq!(ds.right().is_some(), true);
        ds.seek("Welcome!".len() as isize);
        assert_eq!(ds.left().is_some(), true);
        assert_eq!(ds.right(), None);
    }
}