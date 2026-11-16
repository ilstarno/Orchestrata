use std::sync::OnceLock;

use super::{LeadExampleCatalog, LeadPreview, Outcome, PreviewError, Reason, example::Example};

const EXAMPLES: &str = include_str!("../../examples/lead-to-contact-v1.json");
const ID_PREFIX: &str = "L-";
const CONTACT_PREFIX: &str = "CT-";
const SAMPLE_DOMAIN_SUFFIX: &str = ".invalid";

static CATALOG: OnceLock<Result<Vec<Example>, PreviewError>> = OnceLock::new();

pub fn preview(lead_id: &str) -> Result<LeadPreview, PreviewError> {
    if !is_example_id(lead_id) {
        return Err(PreviewError::UnknownExample);
    }
    let examples = examples()?;
    let index = examples
        .binary_search_by(|example| example.lead.id.as_str().cmp(lead_id))
        .map_err(|_| PreviewError::UnknownExample)?;
    Ok(assess(&examples[index]))
}

/// Lists only built-in identifiers, not the example inputs.
pub fn example_catalog() -> Result<LeadExampleCatalog, PreviewError> {
    Ok(LeadExampleCatalog {
        synthetic: true,
        lead_ids: examples()?
            .iter()
            .map(|example| example.lead.id.clone())
            .collect(),
    })
}

fn examples() -> Result<&'static [Example], PreviewError> {
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .map(Vec::as_slice)
        .map_err(|error| *error)
}

fn load_catalog() -> Result<Vec<Example>, PreviewError> {
    let mut entries: Vec<Example> =
        serde_json::from_str(EXAMPLES).map_err(|_| PreviewError::ExamplesUnavailable)?;
    entries.sort_unstable_by(|left, right| left.lead.id.cmp(&right.lead.id));
    if entries.is_empty()
        || entries.iter().any(|entry| !is_example_id(&entry.lead.id))
        || entries
            .windows(2)
            .any(|pair| pair[0].lead.id == pair[1].lead.id)
    {
        return Err(PreviewError::ExamplesUnavailable);
    }
    Ok(entries)
}

fn is_example_id(value: &str) -> bool {
    value
        .strip_prefix(ID_PREFIX)
        .is_some_and(|suffix| suffix.len() == 3 && suffix.bytes().all(|byte| byte.is_ascii_digit()))
}

fn assess(example: &Example) -> LeadPreview {
    let lead = &example.lead;
    let mut result = LeadPreview {
        lead_id: lead.id.clone(),
        outcome: Outcome::Invalid,
        reason: Reason::Invalid,
        assigned_representative_id: None,
        contact_id: None,
    };
    let email = normalize_email(&lead.email);
    if !is_synthetic_address(&email)
        || lead.company.trim().is_empty()
        || lead.account.trim().is_empty()
    {
        return result;
    }

    let mut matches = example
        .contacts
        .iter()
        .filter(|contact| normalize_email(&contact.email) == email);
    let existing = matches.next();
    if matches.next().is_some() || existing.is_some_and(|contact| contact.account != lead.account) {
        result.outcome = Outcome::Review;
        result.reason = Reason::Ambiguous;
        return result;
    }

    let eligible = example.representatives.iter().filter(|rep| {
        rep.available && rep.territory == lead.territory && rep.language == lead.language
    });
    let retained =
        existing.and_then(|contact| eligible.clone().find(|rep| rep.id == contact.owner));
    let owner = retained.or_else(|| eligible.min_by_key(|rep| (rep.workload, rep.id.as_str())));
    let Some(owner) = owner else {
        result.outcome = Outcome::Unassigned;
        result.reason = Reason::Unassigned;
        return result;
    };
    result.assigned_representative_id = Some(owner.id.clone());
    let qualification = &lead.qualification;
    if !(qualification.need && qualification.person && qualification.next_step) {
        result.outcome = Outcome::Held;
        result.reason = Reason::Unqualified;
        return result;
    }

    if let Some(contact) = existing {
        result.outcome = Outcome::Reused;
        result.reason = Reason::Existing;
        result.contact_id = Some(contact.id.clone());
    } else {
        result.outcome = Outcome::Created;
        result.reason = Reason::New;
        result.contact_id = lead
            .id
            .strip_prefix(ID_PREFIX)
            .map(|suffix| format!("{CONTACT_PREFIX}{suffix}"));
    }
    result
}

fn normalize_email(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn is_synthetic_address(value: &str) -> bool {
    value.split_once('@').is_some_and(|(name, domain)| {
        name.starts_with("sample-")
            && name
                .bytes()
                .all(|byte| byte.is_ascii_graphic() && byte != b'@')
            && domain
                .strip_suffix(SAMPLE_DOMAIN_SUFFIX)
                .is_some_and(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && !label.starts_with('-')
                        && !label.ends_with('-')
                        && label.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                        })
                })
    })
}
