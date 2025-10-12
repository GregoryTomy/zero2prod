use unicode_segmentation::UnicodeSegmentation;

pub struct SubscriberName(String);

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

impl SubscriberName {
    // Returns an instance of SubscriberName if the input satisfies all
    // our validation constraints. Panics otherwise]
    pub fn parse(string: String) -> SubscriberName {
        let is_empty_or_whitespace = string.trim().is_empty();

        let is_too_long = string.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            string.chars().any(|s| forbidden_characters.contains(&s));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name", string)
        } else {
            Self(string)
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
