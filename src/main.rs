use std::ops::Range;

use glob::glob;
use markdown::{mdast::Node, unist::Position};
use sqruff_lib::core::{config::FluffConfig, linter::core::Linter};

use crate::commands::Cli;
use clap::Parser;

mod commands;
#[cfg(test)]
mod tests;

fn main() {
    let args = Cli::parse();

    let lnt = get_linter(args.config.unwrap_or(String::from("config.cfg")));

    let mut paths = Vec::new();

    let mut files_checked = 0;
    let mut issues_found = 0;

    for arg_path in args.paths {
        for entry in glob(&arg_path).unwrap() {
            paths.push(entry.unwrap());
        }
    }

    for path in paths {
        let content = std::fs::read_to_string(&path).unwrap();

        if !content.contains("```sql") {
            continue;
        }

        files_checked += 1;

        let process_result = process_content(
            content,
            &lnt,
            matches!(args.command, commands::Commands::Fix),
            Some(&path.display().to_string()),
        );

        issues_found += process_result.issues_found;

        if process_result.fixed_sql.is_some() {
            std::fs::write(&path, process_result.fixed_sql.unwrap()).unwrap();
        }
    }

    println!("Files checked: {files_checked}, Issues found: {issues_found}.");

    if issues_found > 0 {
        std::process::exit(1);
    }
}

fn process_content(
    content: String,
    linter: &Linter,
    fix: bool,
    file_path: Option<&String>,
) -> ContentProcessResult {
    let md = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();

    let mut fixed_code_blocks = Vec::<SQLCodeBlockToFix>::new();

    let mut issues_found = 0;

    for c in md.children().unwrap() {
        if let Node::Code(code_block) = c {
            if code_block.lang == None {
                continue;
            }

            if code_block.lang.clone().unwrap() != "sql" {
                continue;
            }

            let result = check_for_sql_linting_issues(&linter, code_block.value.as_str(), fix);
            if result.issues.is_empty() {
                continue;
            }

            for issue in result.issues {
                let sql_with_issue = &code_block.value[issue.source_slice];

                let code_block_position = code_block
                    .position
                    .clone()
                    .expect("position for code block should not be empty");

                let line_with_issue = code_block_position.start.line + issue.line;
                println!(
                    "{}:{line_with_issue} - {} ({sql_with_issue})",
                    file_path.unwrap_or(&String::from("")),
                    issue.message
                );

                issues_found += 1;

                println!("{}", code_block.value);
            }

            if result.fixed_sql != None {
                let fixed_sql = result.fixed_sql.unwrap();
                if fixed_sql.len() != code_block.value.len() {
                    panic!("TODO: Fixed SQL length does not match original SQL length.",);
                }

                let code_block_position = code_block.position.clone().unwrap();

                let code_block_with_lang = content
                    [code_block_position.start.offset..code_block_position.end.offset]
                    .to_string();

                let code_block_lines: Vec<String> = code_block_with_lang
                    .lines()
                    .map(|x| x.to_string())
                    .collect();

                let mut new_line_char = "\n";
                if &code_block_with_lang[code_block_lines[0].len()..code_block_lines[0].len() + 2]
                    == "\r\n"
                {
                    new_line_char = "\r\n";
                }

                let code_block_close_open_line_length =
                    code_block_lines[0].len() + new_line_char.len();
                let code_block_close_line_length =
                    code_block_lines[code_block_lines.len() - 1].len();

                let mut code_block_position_to_modify = code_block.position.clone().unwrap();

                code_block_position_to_modify.start.offset += code_block_close_open_line_length;
                code_block_position_to_modify.end.offset -= code_block_close_line_length;

                fixed_code_blocks.push(SQLCodeBlockToFix {
                    position: code_block_position_to_modify,
                    sql: fixed_sql + new_line_char,
                });
            }
        }
    }

    let mut fixed_content = None::<String>;

    if !fixed_code_blocks.is_empty() {
        let mut fixed_content_internal = content.clone();
        for fixed_code_block in fixed_code_blocks {
            let start: usize = fixed_code_block.position.start.offset;
            let end = fixed_code_block.position.end.offset;

            fixed_content_internal.replace_range(start..end, &fixed_code_block.sql);
        }

        fixed_content = Some(fixed_content_internal);
    }

    return ContentProcessResult {
        issues_found,
        fixed_sql: fixed_content,
    };
}

fn get_linter(config_path: String) -> Linter {
    let read_file = std::fs::read_to_string(config_path).unwrap();
    let config = FluffConfig::from_source(&read_file, None);

    let lnt = Linter::new(config, None, None, false);
    return lnt;
}

fn check_for_sql_linting_issues(linter: &Linter, sql: &str, fix: bool) -> SQLLintResult {
    let result = linter.lint_string(sql, None, fix);

    let mut is_sql_fixed = false;
    let mut fixed = None::<String>;
    if fix {
        let fixed_sql = result.clone().fix_string();

        is_sql_fixed = sql != fixed_sql;
        if is_sql_fixed {
            // dbg!(&fixed_sql);
            fixed = Some(fixed_sql);
        }
    }

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

    return SQLLintResult {
        issues,
        fixed_sql: if is_sql_fixed { fixed } else { None },
    };
}

struct SQLLintError {
    pub message: String,
    pub source_slice: Range<usize>,
    pub line: usize,
}

struct SQLLintResult {
    pub issues: Vec<SQLLintError>,
    pub fixed_sql: Option<String>,
}

struct SQLCodeBlockToFix {
    pub position: Position,
    pub sql: String,
}

struct ContentProcessResult {
    pub issues_found: u32,
    pub fixed_sql: Option<String>,
}
