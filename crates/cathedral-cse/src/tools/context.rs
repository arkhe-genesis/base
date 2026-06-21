pub struct ToolContext {
    pub workspace_dir: String,
}

impl ToolContext {
    pub fn new(workspace_dir: String) -> Self {
        Self { workspace_dir }
    }
}
