use crate::{
    error::RuleSyntaxError,
    rule::{ParsedRules, RuleGroup, RulePart, validate_part},
};

fn assert_part_err(part: RulePart, fragment: &str, check: fn(RuleSyntaxError) -> bool) {
    let err = validate_part(part, fragment).unwrap_err();
    assert!(check(err.clone()), "unexpected error: {err}");
}

fn assert_part_ok(part: RulePart, fragment: &str) {
    validate_part(part, fragment).unwrap();
}

fn assert_rule_err(rule: &str, check: fn(RuleSyntaxError) -> bool) {
    let groups = vec![RuleGroup {
        name: String::new(),
        rule: vec![rule.to_owned()],
        description: String::new(),
    }];
    let err = ParsedRules::try_from(groups.as_slice()).unwrap_err();
    assert!(check(err.clone()), "unexpected parse error: {err}");
}

fn assert_structure_err(rule: &str, check: fn(RuleSyntaxError) -> bool) {
    let groups = vec![RuleGroup {
        name: String::new(),
        rule: vec![rule.to_owned()],
        description: String::new(),
    }];
    let parsed = ParsedRules::try_from(groups.as_slice()).unwrap();
    let err = parsed.check_structure().unwrap_err();
    assert!(check(err.clone()), "unexpected structure error: {err}");
}

#[test]
fn validate_part_input_errors() {
    assert_part_err(RulePart::Input, "", |e| matches!(e, RuleSyntaxError::EmptyInput(..)));
    assert_part_err(RulePart::Input, "§", |e| matches!(e, RuleSyntaxError::UnknownCharacter(..)));
    assert_part_err(RulePart::Input, "#", |e| matches!(e, RuleSyntaxError::WordBoundLoc(..)));
    assert_part_err(RulePart::Input, "(C)", |e| matches!(e, RuleSyntaxError::OptLocError(..)));
    assert_part_err(RulePart::Input, "VO:[+bogus]", |e| matches!(e, RuleSyntaxError::UnknownFeature(..)));
    assert_part_ok(RulePart::Input, "p");
    assert_part_ok(RulePart::Input, "{p,t,k}");
}

#[test]
fn validate_part_output_errors() {
    assert_part_err(RulePart::Output, "", |e| matches!(e, RuleSyntaxError::EmptyOutput(..)));
    assert_part_err(RulePart::Output, "#", |e| matches!(e, RuleSyntaxError::UnknownCharacter(..)));
    assert_part_err(RulePart::Output, "-p", |e| matches!(e, RuleSyntaxError::BadNegationOutput(..)));
    assert_part_ok(RulePart::Output, "e");
    assert_part_ok(RulePart::Output, "&");
}

#[test]
fn validate_part_context_errors() {
    assert_part_err(RulePart::Context, "", |e| matches!(e, RuleSyntaxError::ExpectedUnderline(..)));
    assert_part_err(RulePart::Context, "a", |e| matches!(e, RuleSyntaxError::ExpectedUnderline(..)));
    assert_part_err(RulePart::Context, "VO:[+bogus]", |e| matches!(e, RuleSyntaxError::UnknownFeature(..)));
    assert_part_ok(RulePart::Context, "#_");
    assert_part_ok(RulePart::Context, "_#");
}

#[test]
fn validate_part_exception_matches_context_grammar() {
    assert_part_err(RulePart::Exception, "", |e| matches!(e, RuleSyntaxError::ExpectedUnderline(..)));
    assert_part_ok(RulePart::Exception, "V_");
}

#[test]
fn validate_part_rejects_leftover_tokens() {
    assert_part_err(RulePart::Input, "p > e", |e| matches!(e, RuleSyntaxError::ExpectedEndLine(..)));
    assert_part_err(RulePart::Context, "#_ / a", |e| matches!(e, RuleSyntaxError::ExpectedEndLine(..)));
}

#[test]
fn check_structure_condensed_io() {
    assert_structure_err("a, u > e, y, x / #_", |e| matches!(e, RuleSyntaxError::UnbalancedRuleIO(..)));
}

#[test]
fn check_structure_insert_delete() {
    assert_structure_err("* > *", |e| matches!(e, RuleSyntaxError::InsertDelete(..)));
    assert_structure_err("∅ > ∅", |e| matches!(e, RuleSyntaxError::InsertDelete(..)));
}

#[test]
fn check_structure_ok_for_balanced_rule() {
    let groups = vec![RuleGroup {
        name: String::new(),
        rule: vec!["p > e / #_".to_owned()],
        description: String::new(),
    }];
    let parsed = ParsedRules::try_from(groups.as_slice()).unwrap();
    parsed.check_structure().unwrap();
}

#[test]
fn whole_rule_empty_sides() {
    assert_rule_err("> e", |e| matches!(e, RuleSyntaxError::UnknownCharacter(..)));
    assert_rule_err("p >", |e| matches!(e, RuleSyntaxError::EmptyOutput(..)));
    assert_rule_err("p > e /", |e| matches!(e, RuleSyntaxError::EmptyEnv(..) | RuleSyntaxError::ExpectedUnderline(..)));
}
