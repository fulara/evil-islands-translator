use std::collections::VecDeque;
use std::process::Command;

fn google_auth_token() -> String {
    let output = Command::new("cmd")
        .args(&[
            "/C",
            "gcloud",
            "auth",
            "application-default",
            "print-access-token",
        ])
        .output()
        .expect("gcloud failed.")
        .stdout;
    String::from_utf8(output)
        .expect("invalid utf8?")
        .trim()
        .to_string()
}

const NEWLINE_MARKER: &str = " ź ";

struct PreparedTranslateInput {
    text: String,
    comments: VecDeque<(usize, String)>,
}

impl PreparedTranslateInput {
    fn new(text: &str) -> Self {
        let mut sanitized = String::new();
        let mut comments = VecDeque::new();
        for (line_no, line) in text.lines().enumerate() {
            if line.starts_with("#") {
                comments.push_back((line_no, line.to_string()));
            } else {
                if line_no != 0 {
                    sanitized.push_str(NEWLINE_MARKER);
                }
                sanitized.push_str(line);
            }
        }

        PreparedTranslateInput {
            text: sanitized,
            comments,
        }
    }

    fn reverse(mut self, text: &str) -> String {
        let lined = text.replace(NEWLINE_MARKER, "\n");
        let mut text = String::new();
        let mut line_no = 0;
        let mut lines = lined.lines();
        loop {
            if let Some((comment_line, comment)) = self.comments.front() {
                if *comment_line == line_no {
                    text.push_str(comment);
                    text.push_str("\n");
                    line_no += 1;
                    self.comments.pop_front();
                    continue;
                }
            }

            if let Some(line) = lines.next() {
                text.push_str(line);
                text.push_str("\n");
                line_no += 1;
            } else {
                break;
            }
        }
        text.pop();
        text
    }
}

fn translate(token: &str, text: &str) -> String {
    let sanitized = PreparedTranslateInput::new(text);
    let body = format!(
        r#"{{
  "q": "{}",
  "source": "ru",
  "target": "en",
  "format": "text"
}}"#,
        sanitized.text
    );
    std::fs::write("request.json", body).expect("couldn't write request.json");
    let mut cmd = Command::new("curl");
    cmd.args(&[
        "-X",
        "POST",
        "-H",
        &format!("Authorization: Bearer {}", token),
        "-H",
        "Content-Type: application/json; charset=utf-8",
        "-d",
        "@request.json",
        "https://translation.googleapis.com/language/translate/v2",
    ]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let response = String::from_utf8(output.stdout).expect("curl failed");
    // Eh, you can say that opening Cargo.toml and adding serde_json was too much.
    let line = response
        .lines()
        .find(|line| line.contains("translatedText"))
        .expect("response did not contain translatedText?");
    let translated = line
        .strip_prefix(r#"        "translatedText": ""#)
        .expect("prefix not found")
        .strip_suffix("\"")
        .expect("suffix not found")
        .to_string();

    sanitized.reverse(&translated)
}

fn main() {
    let token = google_auth_token();
    let response = translate(
        &token,
        r#"заброшенная земля
#potato
заброшенная земля"#,
    );
    println!("Hello, world! {}", response);
}

#[cfg(test)]
mod tests {
    use crate::PreparedTranslateInput;

    #[test]
    fn prepare_input() {
        let test_input = r#"заброшенная земля
#potato
заброшенная земля"#;
        let prepared = PreparedTranslateInput::new(&test_input);
        let exact_response = prepared.text.clone();
        assert_eq!(test_input, prepared.reverse(&exact_response));
    }
}
