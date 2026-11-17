use orchestrata_application::lead_conversion::{ConversionError, ReplayLedger, WriteIntent};
use orchestrata_application::lead_preview::{Outcome, PreviewError};

#[test]
fn duplicate_concurrent_requests_return_one_contact_and_one_event() {
    let ledger = ReplayLedger::default();
    std::thread::scope(|scope| {
        let tasks: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| ledger.apply("L-001", WriteIntent::ApplyToFixture).unwrap()))
            .collect();
        for task in tasks {
            let event = task.join().unwrap();
            assert_eq!(event.lead_id(), "L-001");
            assert_eq!(event.contact_id(), "CT-001");
            assert_eq!(event.outcome(), Outcome::Created);
        }
    });
    assert_eq!(ledger.events().unwrap().len(), 1);
    assert_eq!(ledger.created_contact_count().unwrap(), 1);
}

#[test]
fn denied_unknown_held_and_ambiguous_requests_never_write() {
    let ledger = ReplayLedger::default();
    assert_eq!(
        ledger.apply("L-001", WriteIntent::PreviewOnly),
        Err(ConversionError::ExplicitIntentRequired)
    );
    assert_eq!(
        ledger.apply("unknown", WriteIntent::ApplyToFixture),
        Err(ConversionError::Preview(PreviewError::UnknownExample))
    );
    assert_eq!(
        ledger.apply("L-003", WriteIntent::ApplyToFixture),
        Err(ConversionError::NotConvertible(Outcome::Held))
    );
    assert_eq!(
        ledger.apply("L-008", WriteIntent::ApplyToFixture),
        Err(ConversionError::NotConvertible(Outcome::Review))
    );
    assert!(ledger.events().unwrap().is_empty());
    assert_eq!(ledger.created_contact_count().unwrap(), 0);
}

#[test]
fn linking_an_existing_contact_reuses_the_preview_and_records_once() {
    let ledger = ReplayLedger::default();
    let event = ledger.apply("L-002", WriteIntent::ApplyToFixture).unwrap();
    assert_eq!(event.outcome(), Outcome::Reused);
    let expected = orchestrata_application::lead_preview::preview("L-002").unwrap();
    assert_eq!(Some(event.contact_id()), expected.contact_id.as_deref());
    assert_eq!(
        ledger.apply("L-002", WriteIntent::ApplyToFixture).unwrap(),
        event
    );
    assert_eq!(ledger.events().unwrap(), vec![event]);
    assert_eq!(ledger.created_contact_count().unwrap(), 0);
}
