#[derive(Debug)]
pub struct CreateTodoInput {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct UpdateTodoInput {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
}
