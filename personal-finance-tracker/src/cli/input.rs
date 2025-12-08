/// 极简的单行输入编辑器（如果后面要做可编辑表单可以复用）
#[derive(Default, Clone)]
pub struct LineEdit {
    pub value: String,
    pub cursor: usize,
    pub password: bool,
}

impl LineEdit {
    pub fn set(&mut self, s: impl Into<String>) {
        self.value = s.into();
        self.cursor = self.value.len();
    }
    pub fn push(&mut self, ch: char) {
        self.value.insert(self.cursor, ch);
        self.cursor += 1;
    }
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }
    pub fn delete(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }
    pub fn left(&mut self) {
        if self.cursor > 0 { self.cursor -= 1; }
    }
    pub fn right(&mut self) {
        if self.cursor < self.value.len() { self.cursor += 1; }
    }
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
    pub fn rendered(&self) -> String {
        if self.password { "*".repeat(self.value.len()) } else { self.value.clone() }
    }
}
