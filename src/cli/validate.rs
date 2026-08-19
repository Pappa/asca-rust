use std::io;

use asca::{
    error::RuleSyntaxError,
    rule::{ParsedRules, RuleGroup, RulePart, validate_part},
};

use crate::cli::{parse, util::{self, RULE_FILE_EXT}};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub(crate) enum ValidateField {
    Input,
    Output,
    Context,
    Exception,
}

impl ValidateField {
    fn to_rule_part(self) -> RulePart {
        match self {
            Self::Input => RulePart::Input,
            Self::Output => RulePart::Output,
            Self::Context => RulePart::Context,
            Self::Exception => RulePart::Exception,
        }
    }
}

pub(crate) fn run(
    rules: Option<std::path::PathBuf>,
    rule: Option<String>,
    field: Option<ValidateField>,
    fragment: Option<String>,
) -> io::Result<()> {
  match (field, fragment, rule) {
        (Some(field_kind), Some(fragment), None) => validate_field(field_kind, &fragment),
        (None, None, Some(rule)) => validate_rule_line(&rule),
        (None, None, None) => {
            let rules = parse::parse_rsca(
                &util::validate_or_get_path(rules.as_deref(), &[RULE_FILE_EXT, "txt"], "rule")?,
            )?;
            validate_rule_groups(&rules)
        },
        _ => Err(io::Error::other(
            "asca: use -r for a rule file, -s for one rule line, or -f with a fragment",
        )),
    }
}

fn validate_field(field: ValidateField, fragment: &str) -> io::Result<()> {
    match validate_part(field.to_rule_part(), fragment.trim()) {
        Ok(()) => Ok(()),
        Err(e) => Err(format_rule_error(e, fragment)),
    }
}

fn validate_rule_line(rule: &str) -> io::Result<()> {
    let groups = vec![RuleGroup {
        name: "validate".into(),
        rule: vec![rule.trim().into()],
        description: String::new(),
    }];
    validate_rule_groups(&groups)
}

fn validate_rule_groups(groups: &[RuleGroup]) -> io::Result<()> {
    match ParsedRules::try_from(groups) {
        Ok(parsed) => match parsed.check_structure() {
            Ok(()) => Ok(()),
            Err(e) => Err(format_rule_error_for_groups(e, groups)),
        },
        Err(e) => Err(format_rule_error_for_groups(e, groups)),
    }
}

fn format_rule_error_for_groups(err: RuleSyntaxError, groups: &[RuleGroup]) -> io::Error {
    io::Error::other(err.format(groups))
}

fn format_rule_error(err: RuleSyntaxError, fragment: &str) -> io::Error {
    let groups = vec![RuleGroup {
        name: String::new(),
        rule: vec![fragment.to_owned()],
        description: String::new(),
    }];
    format_rule_error_for_groups(err, &groups)
}
