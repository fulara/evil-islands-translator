use serde::Deserialize;
use std::collections::{VecDeque, HashMap};
use std::process::Command;
use structopt::StructOpt;

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseTranslation {
    translated_text: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    data: HashMap<String, Vec<ResponseTranslation>>,
}

fn translate(token: &str, text: &str) -> String {

    let sanitized = PreparedTranslateInput::new(text);
    let request = format!(
        r#"{{
  "q": "{}",
  "source": "ru",
  "target": "en",
  "format": "text"
}}"#,
        sanitized.text
    );
    std::fs::write("request.json", request).expect("couldn't write request.json");
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
    let response: Response = serde_json::from_str(&response).expect("Failed to deserialize");
    sanitized.reverse(&response.data.get("translations").unwrap()[0].translated_text)
}

#[derive(StructOpt)]
struct VerifyOptions {}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab_case")]
enum Options {
    Verify(VerifyOptions),
}

fn main() {
    let opt = Options::from_args();
    match opt {
        Options::Verify(opts) => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::{PreparedTranslateInput, Response};

    #[test]
    fn prepare_input() {
        let test_input = r#"заброшенная земля
#potato
заброшенная земля"#;
        let prepared = PreparedTranslateInput::new(&test_input);
        let exact_response = prepared.text.clone();
        assert_eq!(test_input, prepared.reverse(&exact_response));
    }

    #[test]
    fn deser_response_test() {
        let response = r#"{
  "data": {
    "translations": [
      {
        "translatedText": "Gipat"
      }
    ]
  }
}"#;
        let response: Response = serde_json::from_str(response).unwrap();
    }
}
