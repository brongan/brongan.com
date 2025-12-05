#[derive(Debug, Clone)]
pub struct Screen(pub [[bool; 64]; 32]);

impl Default for Screen {
    fn default() -> Self {
        Self([[false; 64]; 32])
    }
}
