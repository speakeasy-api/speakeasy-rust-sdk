use regex::Captures;

pub fn get_first_capture<'a>(caps: &'a Captures) -> Option<&'a str> {
    for i in 1..caps.len() {
        if let Some(c) = caps.get(i) {
            return Some(c.as_str());
        }
    }

    None
}
