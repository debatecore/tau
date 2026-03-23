#[cfg(tests)]
mod tests {
 use axum ::{
   body :: Body,
   http :: {Request, StatusCode},
   routing :: get,
   Router,
 };
  use tower :: ServiceExt; // for 'oneshot'
  use uuid :: Uuid; 
  // Note: You will need to mock your App State (Database Pool) and Auth headers 
  // depending on how your existing tests are set up.

  // A helper to create a mock router for testing the endpoint
  fn app () -> Router {
      // Build your router with mock state here
      // Router::new().route("/user/:id/tournaments/:tournament_id/permissions", get(check_permission_endpoint)).with_state(mock_pool)
    todo!("initialize your router with mock dependencies")
  }
  #[tokio::test]
  async fn test_check_permission_success() {
      let app = app();
      let user_id = Uuid::now_v7();
      let tournament_id = Uuid::now_v7();

      // simulating a request for a valid permission
      let uri = format!("/user/{}/tournaments/{}/permissions?permission_name=FakePermission", user_id, tournament_id)

      let request = Request::builder()
            .uri(uri)
            // .header("Authorization", "Bearer mock_token") // Add your auth mock
            .body(Body::empty())
            .unwrap();

      let response = app.oneshot(request).await.unwrap();

    // Assertions 
  }
}




