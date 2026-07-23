use anyhow::{Error, anyhow};
use derive_more::{Deref, Display};
use regex::Regex;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deref, Deserialize, Display, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct JiraIssueProject(String);

#[derive(Debug, Clone, Deserialize, Display, JsonSchema, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
#[display("{project}-{id}")]
pub struct JiraIssueKey {
    pub(super) project: JiraIssueProject,
    id: u32,
}

impl From<JiraIssueKey> for String {
    fn from(key: JiraIssueKey) -> String {
        key.to_string()
    }
}

impl TryFrom<String> for JiraIssueKey {
    type Error = Error;
    fn try_from(key: String) -> Result<Self, Self::Error> {
        key.parse()
    }
}

impl FromStr for JiraIssueKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^(?<proj>[A-Z][A-Z0-9]+)-(?<id>[1-9][0-9]*)$")?;
        if let Some(caps) = re.captures(s) {
            let project = JiraIssueProject(caps["proj"].to_owned());
            let id = caps["id"].parse().unwrap();
            Ok(Self { project, id })
        } else {
            Err(anyhow!("expected format like PROJ-123"))
        }
    }
}

impl FromStr for JiraIssueProject {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^[A-Z][A-Z0-9]+$")?;
        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(anyhow!("expected format like PROJ"))
        }
    }
}

impl JiraIssueKey {
    pub(super) fn is_allowed(&self, allowed: &[JiraIssueProject]) -> bool {
        allowed.iter().any(|allowed| &self.project == allowed)
    }
}
