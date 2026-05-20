/// Parser for Mermaid gitGraph syntax.
///
/// Supported grammar (faithful port of gitGraphDb.ts):
///   gitGraph [LR|TB|BT]
///       commit [id: "id"] [msg: "msg"] [tag: "tag"] [type: NORMAL|REVERSE|HIGHLIGHT]
///       branch <name> [order: N]
///       checkout <name>
///       merge <name> [id: "id"] [tag: "tag"] [type: NORMAL|REVERSE|HIGHLIGHT]
///
/// The DB builds commits and branches in insertion order.
use std::collections::HashMap;

// ── Commit type constants (mirrors gitGraphTypes.ts) ────────────────────────

pub const COMMIT_NORMAL: u8 = 0;
pub const COMMIT_REVERSE: u8 = 1;
pub const COMMIT_HIGHLIGHT: u8 = 2;
pub const COMMIT_MERGE: u8 = 3;
pub const COMMIT_CHERRY_PICK: u8 = 4;

// ── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub seq: usize,
    pub commit_type: u8,
    pub tags: Vec<String>,
    pub parents: Vec<String>,
    pub branch: String,
    pub custom_type: Option<u8>,
    pub custom_id: bool,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct GitGraphDiagram {
    pub direction: DiagramDirection,
    /// Commits in insertion order (keyed by id)
    pub commits: Vec<Commit>,
    /// Branches in creation order
    pub branches: Vec<Branch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagramDirection {
    #[default]
    LR,
    TB,
    BT,
}

// ── Parser state ─────────────────────────────────────────────────────────────

struct DbState {
    commits: Vec<Commit>,
    commit_map: HashMap<String, usize>, // id -> index in commits
    branches: Vec<Branch>,
    branch_index: HashMap<String, usize>, // name -> index in branches
    current_branch: String,
    /// id of the HEAD commit on each branch (None = empty branch)
    branch_head: HashMap<String, Option<String>>,
    seq: usize,
    id_counter: usize,
    direction: DiagramDirection,
}

impl DbState {
    fn new() -> Self {
        let main_branch = "main".to_string();
        let mut branch_index = HashMap::new();
        branch_index.insert(main_branch.clone(), 0usize);
        let mut branch_head = HashMap::new();
        branch_head.insert(main_branch.clone(), None);

        DbState {
            commits: Vec::new(),
            commit_map: HashMap::new(),
            branches: vec![Branch {
                name: main_branch.clone(),
            }],
            branch_index,
            current_branch: main_branch,
            branch_head,
            seq: 0,
            id_counter: 0,
            direction: DiagramDirection::LR,
        }
    }

    fn make_id(&mut self) -> String {
        let n = self.id_counter;
        self.id_counter += 1;
        // Deterministic 7-char hex hash matching Mermaid's auto-id format
        let h = (n as u64)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add(0x6c62272e07bb0142);
        format!("{}-{:07x}", n, h & 0xfffffff)
    }

    fn current_head(&self) -> Option<String> {
        self.branch_head
            .get(&self.current_branch)
            .and_then(|h| h.clone())
    }

    fn do_commit(
        &mut self,
        id: Option<String>,
        _msg: String,
        ctype: u8,
        tags: Vec<String>,
        custom_type: Option<u8>,
    ) {
        let custom_id = id.is_some();
        let id = id.unwrap_or_else(|| self.make_id());
        let parents: Vec<String> = self.current_head().into_iter().collect();
        let seq = self.seq;
        self.seq += 1;

        let commit = Commit {
            id: id.clone(),
            seq,
            commit_type: ctype,
            tags,
            parents,
            branch: self.current_branch.clone(),
            custom_type,
            custom_id,
        };
        let idx = self.commits.len();
        self.commit_map.insert(id.clone(), idx);
        self.commits.push(commit);
        *self.branch_head.get_mut(&self.current_branch).unwrap() = Some(id);
    }

    fn do_branch(&mut self, name: String, order: Option<usize>) {
        if self.branch_index.contains_key(&name) {
            return;
        }
        let order = order.unwrap_or(self.branches.len());
        let idx = self.branches.len();
        let _ = order; // parsed but not stored
        self.branches.push(Branch { name: name.clone() });
        self.branch_index.insert(name.clone(), idx);
        // New branch head = current HEAD (branching point)
        let head = self.current_head();
        self.branch_head.insert(name.clone(), head);
        self.current_branch = name;
    }

    fn do_checkout(&mut self, name: &str) {
        if self.branch_index.contains_key(name) {
            self.current_branch = name.to_string();
        }
    }

    fn do_merge(&mut self, branch: &str, id: Option<String>, tags: Vec<String>, mtype: Option<u8>) {
        if !self.branch_index.contains_key(branch) {
            return;
        }
        // The merge commit has two parents: current HEAD and the HEAD of the merged branch
        let source_head = self.branch_head.get(branch).and_then(|h| h.clone());
        let current_head = self.current_head();

        let custom_id = id.is_some();
        let commit_id = id.unwrap_or_else(|| self.make_id());
        let mut parents: Vec<String> = Vec::new();
        if let Some(ch) = current_head {
            parents.push(ch);
        }
        if let Some(sh) = source_head {
            if !parents.contains(&sh) {
                parents.push(sh);
            }
        }

        let seq = self.seq;
        self.seq += 1;
        let commit = Commit {
            id: commit_id.clone(),
            seq,
            commit_type: COMMIT_MERGE,
            tags,
            parents,
            branch: self.current_branch.clone(),
            custom_type: mtype,
            custom_id,
        };
        let idx = self.commits.len();
        self.commit_map.insert(commit_id.clone(), idx);
        self.commits.push(commit);
        *self.branch_head.get_mut(&self.current_branch).unwrap() = Some(commit_id);
    }
}

// ── Public parse entry point ─────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<GitGraphDiagram> {
    let mut state = DbState::new();
    let mut in_graph = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // Header line
        if let Some(stripped) = trimmed.strip_prefix("gitGraph") {
            in_graph = true;
            let rest = stripped.trim();
            // optional direction modifier
            if rest.starts_with("LR") {
                state.direction = DiagramDirection::LR;
            } else if rest.starts_with("TB") {
                state.direction = DiagramDirection::TB;
            } else if rest.starts_with("BT") {
                state.direction = DiagramDirection::BT;
            }
            continue;
        }

        // title keyword — skip (not used by renderer)
        if trimmed.starts_with("title") {
            continue;
        }

        if !in_graph {
            continue;
        }

        if trimmed.starts_with("commit") {
            parse_commit(trimmed, &mut state);
        } else if trimmed.starts_with("branch") {
            parse_branch(trimmed, &mut state);
        } else if trimmed.starts_with("checkout") || trimmed.starts_with("switch") {
            parse_checkout(trimmed, &mut state);
        } else if trimmed.starts_with("merge") {
            parse_merge(trimmed, &mut state);
        }
    }

    crate::error::ParseResult::ok(GitGraphDiagram {
        direction: state.direction,
        commits: state.commits,
        branches: state.branches,
    })
}

// ── Line parsers ─────────────────────────────────────────────────────────────

fn parse_commit(line: &str, state: &mut DbState) {
    let rest = line["commit".len()..].trim();
    let mut id: Option<String> = None;
    let mut msg = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut ctype = COMMIT_NORMAL;
    let mut custom_type: Option<u8> = None;

    parse_options(rest, |key, val| match key {
        "id" => id = Some(val.to_string()),
        "msg" => msg = val.to_string(),
        "tag" => tags.push(val.to_string()),
        "type" => match val {
            "REVERSE" => ctype = COMMIT_REVERSE,
            "HIGHLIGHT" => ctype = COMMIT_HIGHLIGHT,
            "NORMAL" => ctype = COMMIT_NORMAL,
            _ => {}
        },
        _ => {}
    });

    state.do_commit(id, msg, ctype, tags, custom_type.take());
}

fn parse_branch(line: &str, state: &mut DbState) {
    let rest = line["branch".len()..].trim();
    // Branch name is the first token; optional `order: N`
    let mut order: Option<usize> = None;

    // Split into tokens — branch name may contain hyphens/slashes
    // Pattern: branch <name> [order: N]
    let name = if let Some(pos) = rest.find(" order") {
        // Everything before " order" is the branch name
        let n = rest[..pos].trim().to_string();
        // parse order value
        let after = rest[pos..].trim();
        parse_options(after, |key, val| {
            if key == "order" {
                order = val.parse().ok();
            }
        });
        n
    } else {
        rest.to_string()
    };

    if !name.is_empty() {
        state.do_branch(name, order);
    }
}

fn parse_checkout(line: &str, state: &mut DbState) {
    // `checkout <name>` or `switch <name>`
    let rest = if let Some(stripped) = line.strip_prefix("switch") {
        stripped.trim()
    } else {
        line["checkout".len()..].trim()
    };
    state.do_checkout(rest);
}

fn parse_merge(line: &str, state: &mut DbState) {
    let rest = line["merge".len()..].trim();
    let mut id: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut mtype: Option<u8> = None;

    // First token is branch name, rest are options
    let (branch, opts) = split_first_token(rest);

    parse_options(opts, |key, val| match key {
        "id" => id = Some(val.to_string()),
        "tag" => tags.push(val.to_string()),
        "type" => {
            mtype = Some(match val {
                "REVERSE" => COMMIT_REVERSE,
                "HIGHLIGHT" => COMMIT_HIGHLIGHT,
                "MERGE" => COMMIT_MERGE,
                _ => COMMIT_NORMAL,
            });
        }
        _ => {}
    });

    if !branch.is_empty() {
        state.do_merge(branch, id, tags, mtype);
    }
}

// ── Option parsing helpers ────────────────────────────────────────────────────

/// Split a string into (first_word, remainder).
fn split_first_token(s: &str) -> (&str, &str) {
    let s = s.trim();
    if let Some(pos) = s.find(|c: char| c.is_whitespace()) {
        (s[..pos].trim(), s[pos..].trim())
    } else {
        (s, "")
    }
}

/// Walk key-value pairs of the form: `key: "value"` or `key: bareword`
fn parse_options<F: FnMut(&str, &str)>(opts: &str, mut f: F) {
    // Tokenize: find `word:` patterns followed by quoted or bare values
    let mut s = opts.trim();
    while !s.is_empty() {
        // Find the next key (word followed by ':')
        let colon_pos = match s.find(':') {
            Some(p) => p,
            None => break,
        };
        let key = s[..colon_pos].trim();
        s = s[colon_pos + 1..].trim_start();

        // Parse value: either quoted or bare word
        let (val, rest) = if let Some(stripped) = s.strip_prefix('"') {
            // Quoted string
            if let Some(end) = stripped.find('"') {
                (&s[1..end + 1], s[end + 2..].trim_start())
            } else {
                (stripped, "")
            }
        } else {
            // Bare word until whitespace or next colon sequence
            // Stop at whitespace
            if let Some(pos) = s.find(|c: char| c.is_whitespace()) {
                (s[..pos].trim(), s[pos..].trim_start())
            } else {
                (s, "")
            }
        };

        if !key.is_empty() {
            f(key, val);
        }
        s = rest;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_graph() {
        let input = "gitGraph\n    commit\n    branch develop\n    commit\n    checkout main\n    merge develop";
        let d = parse(input).diagram;
        assert_eq!(d.branches.len(), 2);
        // 3 commits: initial, develop commit, merge
        assert_eq!(d.commits.len(), 3);
    }

    #[test]
    fn direction_lr() {
        let input = "gitGraph LR\n    commit";
        let d = parse(input).diagram;
        assert_eq!(d.direction, DiagramDirection::LR);
    }

    #[test]
    fn direction_tb() {
        let input = "gitGraph TB\n    commit";
        let d = parse(input).diagram;
        assert_eq!(d.direction, DiagramDirection::TB);
    }

    #[test]
    fn merge_commit_type() {
        let input =
            "gitGraph\n    commit\n    branch dev\n    commit\n    checkout main\n    merge dev";
        let d = parse(input).diagram;
        let merge = d.commits.last().unwrap();
        assert_eq!(merge.commit_type, COMMIT_MERGE);
        assert_eq!(merge.parents.len(), 2);
    }

    #[test]
    fn sample_diagram() {
        let input = r#"gitGraph
    commit
    branch develop
    commit
    commit
    checkout main
    merge develop
    commit
    branch feature
    commit
    checkout develop
    merge feature"#;
        let d = parse(input).diagram;
        // main, develop, feature
        assert_eq!(d.branches.len(), 3);
        // 7 operations: commit, commit, commit, merge, commit, commit, merge
        assert_eq!(d.commits.len(), 7);
    }
}

#[cfg(test)]
mod tag_test {
    #[test]
    fn tag_with_dot() {
        let input = "gitGraph\n   commit id: \"1\" tag: \"v1.0\"";
        let diag = super::parse(input).diagram;
        let tags: Vec<&str> = diag
            .commits
            .iter()
            .flat_map(|c| c.tags.iter().map(|t| t.as_str()))
            .collect();
        assert_eq!(tags, vec!["v1.0"], "tag should be v1.0 not {:?}", tags);
    }
}
