use super::{ModelsListResponse, ModelsListRow};

#[test]
pub(crate) fn kiss_cov_models_list_response_row_types() {
    let _ = stringify!(ModelsListResponse);
    let _ = stringify!(ModelsListRow);
    let row = ModelsListRow {
        id: "provider/model".into(),
        name: "Model".into(),
    };
    let response = ModelsListResponse {
        data: vec![row],
    };
    assert_eq!(response.data[0].name, "Model");
}
