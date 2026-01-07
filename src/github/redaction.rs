//! Redaction helpers for logs and error strings.

pub fn redact_secrets(input: &str) -> String {
    let mut output = input.to_string();
    output = redact_prefixed_token(&output, "ghp_");
    output = redact_prefixed_token(&output, "github_pat_");
    output = redact_bearer(&output);
    output = redact_url_credentials(&output);
    output
}

fn redact_prefixed_token(input: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(index) = rest.find(prefix) {
        let (before, after_prefix) = rest.split_at(index);
        result.push_str(before);

        let after_prefix = &after_prefix[prefix.len()..];
        let token_len = after_prefix
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .count();

        if token_len > 0 {
            result.push_str("<redacted>");
            rest = &after_prefix[token_len..];
        } else {
            result.push_str(prefix);
            rest = after_prefix;
        }
    }

    result.push_str(rest);
    result
}

fn redact_bearer(input: &str) -> String {
    const PREFIX: &str = "Bearer ";
    let mut result = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(index) = rest.find(PREFIX) {
        let (before, after_prefix) = rest.split_at(index);
        result.push_str(before);

        let after_prefix = &after_prefix[PREFIX.len()..];
        let token_len = after_prefix
            .chars()
            .take_while(|c| !c.is_whitespace())
            .count();

        if token_len > 0 {
            result.push_str("Bearer <redacted>");
            rest = &after_prefix[token_len..];
        } else {
            result.push_str(PREFIX);
            rest = after_prefix;
        }
    }

    result.push_str(rest);
    result
}

fn redact_url_credentials(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(index) = rest.find("://") {
        let (before, after_scheme) = rest.split_at(index + 3);
        result.push_str(before);

        let segment_end = after_scheme
            .find(char::is_whitespace)
            .unwrap_or(after_scheme.len());
        let segment = &after_scheme[..segment_end];

        if let Some(at_index) = segment.find('@') {
            let auth = &segment[..at_index];
            if auth.contains(':') {
                result.push_str("<redacted>@");
                result.push_str(&segment[at_index + 1..]);
                rest = &after_scheme[segment_end..];
                continue;
            }
        }

        result.push_str(segment);
        rest = &after_scheme[segment_end..];
    }

    result.push_str(rest);
    result
}
