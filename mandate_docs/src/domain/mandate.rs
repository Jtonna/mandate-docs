//! Domain types for a mandate. Plain data, no serde, no I/O.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mandate {
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<Rule>,
    pub governs: Vec<GovernedDoc>,
    pub code: Vec<CodeLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub id: String,
    pub description: Option<String>,
    pub kind: RuleKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleKind {
    Script { run: String },
    Agent { prompt: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernedDoc {
    pub doc: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeLink {
    pub path: String,
    pub docs: Vec<String>,
}
