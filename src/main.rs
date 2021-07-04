use encoding::{DecoderTrap, Encoding};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, mem};
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
    line_count_name: VecDeque<(usize, String)>,
}

impl PreparedTranslateInput {
    fn new(names_contents: &[(String, String)]) -> Self {
        // All this just because I could not get the gcloud working on windows in rust? ..
        // Why not just use vm? oh well we are almost there...
        let mut sanitized = String::new();
        let mut comments = VecDeque::new();
        let mut line_count_name = VecDeque::new();
        let mut line_no = 0;
        for (filename, contents) in names_contents {
            let line_no_before = line_no;
            for line in contents.lines() {
                if line.starts_with("#") {
                    comments.push_back((line_no, line.to_string()));
                } else {
                    if line_no != 0 {
                        sanitized.push_str(NEWLINE_MARKER);
                    }
                    sanitized.push_str(line);
                }
                line_no += 1;
            }
            let line_count = line_no - line_no_before;
            line_count_name.push_back((line_count, filename.to_string()));
        }

        PreparedTranslateInput {
            text: sanitized,
            comments,
            line_count_name,
        }
    }

    fn reverse(mut self, text: &str) -> Vec<(String, String)> {
        let lined = text.replace(NEWLINE_MARKER, "\n");
        let mut text = String::new();
        let mut line_no = 0;
        let mut lines = lined.lines();
        let mut current_file_count = 0;
        let mut result = Vec::new();
        loop {
            let (file_original_line_count, filename) = self.line_count_name.front().unwrap();
            if current_file_count == *file_original_line_count {
                text.pop();
                result.push((filename.to_owned(), text.to_string()));
                text.clear();
                current_file_count = 0;
                self.line_count_name.pop_front().unwrap();
                if self.line_count_name.is_empty() {
                    break;
                }
            }

            if let Some((comment_line, comment)) = self.comments.front() {
                if *comment_line == line_no {
                    text.push_str(comment);
                    text.push_str("\n");
                    line_no += 1;
                    current_file_count += 1;
                    self.comments.pop_front();
                    continue;
                }
            }

            if let Some(line) = lines.next() {
                text.push_str(line);
                text.push_str("\n");
                line_no += 1;
                current_file_count += 1;
            } else {
                break;
            }
        }
        result
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    q: String,
    source: String,
    target: String,
    format: String,
}

fn translate_text(token: &str, names_contents: &[(String, String)]) -> Vec<(String, String)> {
    let sanitized = PreparedTranslateInput::new(names_contents);
    let request = Request {
        q: sanitized.text.clone(),
        source: "ru".into(),
        target: "en".into(),
        format: "text".into(),
    };
    std::fs::write("request.json", serde_json::to_string(&request).unwrap())
        .expect("couldn't write request.json");
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
    println!("response we got is: {}", response);
    let response: Response = serde_json::from_str(&response).expect("Failed to deserialize");
    sanitized.reverse(&response.data.get("translations").unwrap()[0].translated_text)
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab_case")]
struct VerifyOptions {}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab_case")]
struct TranslateOptions {
    #[structopt(long)]
    target_lang_key: String,
    #[structopt(long)]
    mod_name: String,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab_case")]
enum Options {
    Verify(VerifyOptions),
    Translate(TranslateOptions),
}

struct Translation {
    path: PathBuf,
    lang_key: String,
}

fn path_to_name_contents(path: &Path) -> (String, String) {
    let mut buff = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buff).unwrap();
    let contents = encoding::decode(&buff, DecoderTrap::Strict, encoding::all::WINDOWS_1251)
        .0
        .unwrap();
    (
        path.file_name().unwrap().to_str().unwrap().to_owned(),
        contents,
    )
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

fn translate(opts: &TranslateOptions) {
    let mut path = Path::new("translate").join(&opts.mod_name);
    let paths: Vec<_> = fs::read_dir(&path).unwrap().map(|de| de.unwrap()).collect();
    let mut baseline = None;
    for p in paths {
        let filename = p.file_name();
        let filename = filename.to_str().unwrap();
        if filename.contains("_ru_") {
            baseline = Some(p.path());
        }
    }
    let baseline = baseline.expect("No russian translation available for chosen mod");
    let target = path.clone().join(
        baseline
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("_ru_", &format!("_{}_", opts.target_lang_key)),
    );
    fs::create_dir_all(&target).expect("Could not create target translation");

    let token = google_auth_token();
    let mut total_length = 0;
    let mut to_translate = Vec::new();
    for f in baseline.read_dir().unwrap() {
        let f = f.unwrap();
        println!("Processing: {:?}", f.file_name());
        let (source_name, source_contents) = path_to_name_contents(&f.path());
        if source_contents.len() == 0 {
            fs::create_dir_all(target.join(f.file_name()));
        } else if !target.join(f.file_name()).exists() {
            total_length += source_contents.len();
            to_translate.push((source_name, source_contents));
            if total_length > 1000 {
                let response = translate_text(&token, &to_translate);
                for (filename, contents) in response {
                    let path = target.join(filename);
                    fs::write(&path, contents).unwrap();
                }
                to_translate.clear();
                total_length = 0;
            }
        }
    }
    if !to_translate.is_empty() {
        let response = translate_text(&token, &to_translate);
        for (filename, contents) in response {
            let path = target.join(filename);
            fs::write(&path, contents).unwrap();
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
        Options::Translate(opts) => {
            translate(&opts);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{PreparedTranslateInput, Response};

    #[test]
    fn prepare_input() {
        let file1 = (
            "abc".to_string(),
            r#"заброшенная земля
#potato
заброшенная земля"#
                .to_string(),
        );
        let file2 = (
            "bca".to_string(),
            r#"green potato
#potato
that's flying"#
                .to_string(),
        );
        let test_input = vec![file1, file2];
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
