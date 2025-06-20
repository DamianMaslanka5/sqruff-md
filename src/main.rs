use std::{
    env::{self},
    ops::Range,
};

use glob::glob;
use markdown::mdast::Node;
use sqruff_lib::core::{config::FluffConfig, linter::core::Linter};

fn main() {
    let cmd_args: Vec<String> = env::args().collect();

    let lnt = get_linter();

    let glob_pattern = cmd_args
        .get(1)
        .unwrap_or(&String::from("./**/*.md"))
        .to_owned();

    let mut paths = Vec::new();

    let mut files_checked = 0;
    let mut issues_found = 0;

    for entry in glob(glob_pattern.as_str()).unwrap() {
        paths.push(entry.unwrap());
    }

    for path in paths {
        let content = std::fs::read_to_string(&path).unwrap();

        if !content.contains("```sql") {
            continue;
        }

        files_checked += 1;

        let md = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();

        for c in md.children().unwrap() {
            if let Node::Code(code_block) = c {
                if code_block.lang == None {
                    continue;
                }

                if code_block.lang.clone().unwrap() == "sql" {
                    let issues = check_for_sql_linting_issues(&lnt, code_block.value.as_str());
                    if !issues.is_empty() {
                        for issue in issues {
                            let sql_with_issue = &code_block.value[issue.source_slice];

                            let code_block_position = code_block
                                .position
                                .clone()
                                .expect("position for code block should not be empty");

                            let line_with_issue = code_block_position.start.line + issue.line;
                            println!(
                                "{}:{line_with_issue} - {} ({sql_with_issue})",
                                path.display(),
                                issue.message
                            );

                            issues_found += 1;
                        }

                        println!("{}", code_block.value);
                    }
                }
            }
        }
    }

    println!("Files checked: {files_checked}, Issues found: {issues_found}.");

    if issues_found > 0 {
        std::process::exit(1);
    }
}

fn get_linter() -> Linter {
    let read_file = std::fs::read_to_string("config.cfg").unwrap();
    let config = FluffConfig::from_source(&read_file, None);

    let lnt = Linter::new(config, None, None, false);
    lnt
}

fn check_for_sql_linting_issues(linter: &Linter, sql: &str) -> Vec<SQLLintError> {
    let result = linter.lint_string(sql, None, false);

    let mut issues = Vec::new();

    for v in result.get_violations(None) {
        // dbg!(&v);

        if v.rule == None {
            // TODO rules is empty when sql is not valid
            // println!("INFO: Skipping violation without rule: {:?}", v);
            continue;
        }
        issues.push(SQLLintError {
            message: v.description,
            source_slice: v.source_slice,
            line: v.line_no,
        });
    }

    return issues;
}

struct SQLLintError {
    pub message: String,
    pub source_slice: Range<usize>,
    pub line: usize,
}
