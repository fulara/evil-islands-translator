use encoding::{DecoderTrap, Encoding};
use itertools::Itertools;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
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

struct Translation {
    path: PathBuf,
    lang_key: String,
}

fn verify_translations(baseline: Translation, translations: &[Translation]) {
    println!("Proceeding to verify translation");
    // encoding::all::WINDOWS_1251.decode()
    let baseline_files: Vec<_> = fs::read_dir(&baseline.path)
        .unwrap()
        .map(|e| e.unwrap())
        .collect();
    let baseline_files: HashMap<_, _> = baseline_files
        .iter()
        .map(|de| {
            let name = de.file_name().to_str().unwrap().to_owned();
            let mut buff = Vec::new();
            let mut file = File::open(de.path()).unwrap().read(&mut buff);
            let decoded = encoding::all::WINDOWS_1251
                .decode(&buff, DecoderTrap::Strict)
                .expect("Decoding failed");
            (
                name,
                decoded.lines().map(|l| l.to_string()).collect::<Vec<_>>(),
            )
        })
        .collect();
    for t in translations {
        println!("Doing stuff on: {}", t.lang_key);
        let files: Vec<_> = fs::read_dir(&t.path).unwrap().map(|e| e.unwrap()).collect();
        let files: HashMap<_, _> = files
            .iter()
            .map(|de| {
                let name = de.file_name().to_str().unwrap().to_owned();
                let content = fs::read_to_string(de.path()).expect("Failed to read file");
                (
                    name,
                    content.lines().map(|l| l.to_string()).collect::<Vec<_>>(),
                )
            })
            .collect();
        if baseline_files.len() != files.len() {
            println!("Mismatch file count in translation: {}", t.lang_key);
            for filename in baseline_files.keys() {
                if !files.contains_key(filename) {
                    println!(
                        "File: {:?} present in base translation(ru) but not in translation({})",
                        filename, t.lang_key
                    )
                }
            }
            for filename in files.keys() {
                if !baseline_files.contains_key(filename) {
                    println!(
                        "File: {:?} present in translation: {} but not in base translation(ru)",
                        filename, t.lang_key
                    )
                }
            }
            panic!("Integrity verification failed.");
        }
    }
}

fn verify() {
    let paths = fs::read_dir("translate").unwrap();
    for path in paths {
        let path = path.unwrap();
        let metadata = path.metadata().unwrap();
        if metadata.is_dir() {
            println!("Will verify: {:?}", path.file_name());
            let paths: Vec<_> = fs::read_dir(path.path())
                .unwrap()
                .filter_map(|rd| {
                    let rd = rd.unwrap();
                    if rd.metadata().unwrap().is_dir() {
                        Some(rd.path())
                    } else {
                        None
                    }
                })
                .collect();
            let mut baseline_translation = None;
            let mut translations = vec![];

            for path in paths.into_iter() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                assert!(
                    file_name.starts_with("texts") && file_name.ends_with("_res"),
                    "Unexpected dir name: {:?}",
                    file_name
                );
                let lang_key = &file_name[file_name.len() - 6..file_name.len() - 4];
                let translation = Translation {
                    path: path.clone(),
                    lang_key: lang_key.to_owned(),
                };
                if lang_key == "ru" {
                    baseline_translation = Some(translation);
                } else {
                    translations.push(translation);
                }
            }
            let baseline_translation =
                baseline_translation.expect("Russian is a base translation its always required");
            println!(
                "There are {} translation available: ru,{}",
                translations.len() + 1,
                translations.iter().map(|t| &t.lang_key).join(",")
            );
            verify_translations(baseline_translation, &translations);
        }
    }
}

fn main() {
    assert!(
        fs::metadata("translate")
            .expect("translate dir does not exists")
            .is_dir(),
        "translate dir should exist in the dir you are running the tool from"
    );
    let opt = Options::from_args();
    match opt {
        Options::Verify(opts) => {
            verify();
        }
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
