use uuid::Uuid;

#[derive(Clone, Debug)]
pub(crate) struct MemoId(String);

impl MemoId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MemoId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Memo {
    #[allow(dead_code)]
    pub(crate) memo_id: MemoId,
    pub(crate) content: String,
    pub(crate) created_at: String,
    #[allow(dead_code)]
    pub(crate) updated_at: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NewMemo {
    pub(crate) content: String,
}

impl NewMemo {
    pub(crate) fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}
