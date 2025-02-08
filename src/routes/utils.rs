pub fn handle_failed_to_get_resource_by_id(e: OmniError) -> Response {
    match e {
        OmniError::ResourceNotFoundError => e.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
