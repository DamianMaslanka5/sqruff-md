use super::*;

fn default_linter() -> Linter {
    return get_linter(String::from("config.cfg"));
}

#[test]
fn fix() {
    let md = "```sql
select 1
```";

    let expected = "```sql
SELECT 1
```";

    let result = process_content(md.to_string(), &default_linter(), true, None);

    assert_eq!(result.fixed_sql.unwrap(), expected);
}

#[test]
fn fix_crlf() {
    let md = "```sql\r\nselect 1\r\n```";

    let expected = "```sql\r\nSELECT 1\r\n```";

    let result = process_content(md.to_string(), &default_linter(), true, None);

    assert_eq!(result.fixed_sql.unwrap(), expected);
}

#[test]
fn unparsable() {
    let md = "```sql
sel 1
```";

    let result = process_content(md.to_string(), &default_linter(), false, None);

    assert_eq!(1, result.unparsable_sql.len());

    assert_eq!("sel 1", result.unparsable_sql.first().unwrap());
}
