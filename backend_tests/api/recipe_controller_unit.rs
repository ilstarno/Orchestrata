use super::*;

#[tokio::test]
async fn keeps_the_field_service_route_contract() {
    let Json(metadata) = metadata(Path("field-service".into())).await.unwrap();
    assert_eq!(metadata.id, "field-service");
    assert_eq!(metadata.inputs.len(), 4);
    assert_eq!(metadata.outputs[0].name, "assigned-technicians");
    assert_eq!(example(Path("missing".into())).await, Err(UNKNOWN_RECIPE));
}
