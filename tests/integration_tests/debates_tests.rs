use reqwest::StatusCode;
use tau::{omni_error::OmniError};

use crate::common::{
    debates_utils::{get_debate, get_id_of_a_new_debate},
    get_response_json,
    test_app::TestApp,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::get_organizer_token,
};

#[tokio::test]
async fn everyone_can_get_debate_details() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id).await?;

    // WHEN
    let response = get_debate(&app, &debate_id, &tournament_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = get_response_json(response).await?;
    assert_eq!(response_body["tournament_id"], tournament_id.to_string());

    Ok(())
}
