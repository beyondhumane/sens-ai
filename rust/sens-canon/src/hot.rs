use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use sens_index::build;
use sens_index::index::{Index, NOBODY, SymbolInfo};
use sens_index::lang::treesitter::{EmitSymbol, collect_leaves, dequote, text};
use sens_index::unity;
use tree_sitter::{Node, Tree};

use crate::verdict::Severity;

const PER_FRAME: [&str; 23] = [
    "Update", "LateUpdate", "FixedUpdate", "OnGUI", "OnAnimatorIK", "OnAnimatorMove", "OnWillRenderObject", "OnPreCull", "OnPreRender", "OnPostRender", "OnRenderImage", "OnRenderObject", "OnDrawGizmos",
    "OnDrawGizmosSelected", "OnTriggerStay", "OnTriggerStay2D", "OnCollisionStay", "OnCollisionStay2D", "OnMouseOver", "OnMouseDrag", "OnStateUpdate", "OnStateMove", "OnStateIK",
];
const LOOPS: [&str; 4] = ["while_statement", "for_statement", "foreach_statement", "do_statement"];
const EVERY_FRAME_WAITS: [&str; 2] = ["WaitForEndOfFrame", "WaitForFixedUpdate"];
const COROUTINE: &str = "IEnumerator";
const NEAR: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reach {
    Frame,
    Calls { count: u32, from: String },
    Far { from: String },
    Cold,
}

impl Reach {
    pub fn graded(&self, declared: Severity) -> Option<Severity> {
        match self {
            Reach::Frame => Some(declared),
            Reach::Calls { .. } if declared == Severity::Block => Some(Severity::Consider),
            Reach::Calls { .. } | Reach::Far { .. } => Some(Severity::Note),
            Reach::Cold => None,
        }
    }
}

pub struct Script<'s> {
    pub path: &'s str,
    pub source: &'s str,
    pub tree: Tree,
    pub symbols: Vec<EmitSymbol>,
    pub components: HashMap<String, &'static str>,
}

impl<'s> Script<'s> {
    pub fn read(index: &Index, path: &'s str, source: &'s str) -> Option<Script<'s>> {
        let (tree, emitted) = build::parse(path, source)?;
        let mut components = index.unity.clone();
        let types: Vec<(&str, &[String])> = emitted.symbols.iter().filter(|symbol| symbol.kind == "class").map(|symbol| (symbol.name.as_str(), symbol.bases.as_slice())).collect();
        unity::derive(&mut components, &types);
        Some(Script { path, source, tree, symbols: emitted.symbols, components })
    }

    pub fn method_at(&self, byte: usize) -> Option<&str> {
        self.methods().filter(|symbol| symbol.start <= byte && byte < symbol.end).max_by_key(|symbol| symbol.start).map(|symbol| symbol.name.as_str())
    }

    fn methods(&self) -> impl Iterator<Item = &EmitSymbol> {
        self.symbols.iter().filter(|symbol| symbol.kind == "method")
    }
}

pub struct Heat {
    nodes: HashMap<String, usize>,
    reach: Vec<Reach>,
}

fn is_method(symbol: &SymbolInfo) -> bool {
    symbol.kind == "method" && symbol.file.ends_with(".cs")
}

fn simple(method: &str) -> &str {
    method.rsplit_once('.').map_or(method, |(_, name)| name)
}

fn runs_every_frame(components: &HashMap<String, &'static str>, method: &str) -> bool {
    method.rsplit_once('.').is_some_and(|(owner, name)| components.contains_key(owner) && PER_FRAME.contains(&name))
}

fn yields_every_frame(node: Node, source: &str, looping: bool) -> bool {
    if node.kind() == "yield_statement" {
        return looping && node.named_child(0).is_some_and(|said| said.kind() == "null_literal" || EVERY_FRAME_WAITS.iter().any(|wait| text(&said, source).contains(wait)));
    }
    let looping = looping || LOOPS.contains(&node.kind());
    let mut cursor = node.walk();
    let children: Vec<Node> = node.named_children(&mut cursor).collect();
    children.into_iter().any(|child| yields_every_frame(child, source, looping))
}

fn frame_coroutines<'e>(tree: &Tree, source: &str, symbols: &'e [EmitSymbol]) -> Vec<&'e str> {
    symbols
        .iter()
        .filter(|symbol| symbol.kind == "method" && symbol.signature.contains(COROUTINE))
        .filter(|symbol| tree.root_node().descendant_for_byte_range(symbol.start, symbol.end).is_some_and(|method| yields_every_frame(method, source, false)))
        .map(|symbol| symbol.name.as_str())
        .collect()
}

fn coroutines_every_frame(index: &Index, script: Option<&Script>) -> HashSet<String> {
    let written = script.map(|script| script.path);
    let files: BTreeSet<&str> = index
        .symbols
        .iter()
        .filter(|symbol| is_method(symbol) && symbol.signature.contains(COROUTINE) && Some(symbol.file.as_str()) != written)
        .map(|symbol| symbol.file.as_str())
        .collect();
    let mut found: HashSet<String> = HashSet::new();
    for file in files {
        let source = std::fs::read_to_string(index.root.join(file)).unwrap_or_default();
        if let Some((tree, emitted)) = build::parse(file, &source) {
            found.extend(frame_coroutines(&tree, &source, &emitted.symbols).into_iter().map(str::to_string));
        }
    }
    if let Some(script) = script {
        found.extend(frame_coroutines(&script.tree, script.source, &script.symbols).into_iter().map(str::to_string));
    }
    found
}

fn spread(edges: &[Vec<(usize, bool)>], roots: &[usize], resolved_only: bool) -> Vec<Option<(u32, usize)>> {
    let mut found: Vec<Option<(u32, usize)>> = vec![None; edges.len()];
    let mut queue: VecDeque<usize> = VecDeque::new();
    for &root in roots {
        found[root] = Some((0, root));
        queue.push_back(root);
    }
    while let Some(at) = queue.pop_front() {
        let Some((depth, root)) = found[at] else { continue };
        for &(next, resolved) in &edges[at] {
            if (resolved || !resolved_only) && found[next].is_none() {
                found[next] = Some((depth + 1, root));
                queue.push_back(next);
            }
        }
    }
    found
}

fn script_calls(script: &Script, nodes: &HashMap<String, usize>, owners: &HashMap<&str, Vec<usize>>, edges: &mut [Vec<(usize, bool)>]) {
    let declared: HashSet<usize> = script.symbols.iter().map(|symbol| symbol.name_start).collect();
    let mut leaves = Vec::new();
    collect_leaves(script.tree.root_node(), &mut leaves);
    for leaf in leaves.into_iter().filter(|leaf| !declared.contains(&leaf.start_byte())) {
        let raw = text(&leaf, script.source);
        let Some(targets) = owners.get(raw).or_else(|| dequote(raw).and_then(|inner| owners.get(inner))) else { continue };
        let Some(&from) = script.method_at(leaf.start_byte()).and_then(|method| nodes.get(method)) else { continue };
        for &to in targets.iter().filter(|&&to| to != from) {
            edges[from].push((to, targets.len() == 1));
        }
    }
}

impl Heat {
    pub fn of(index: &Index, script: Option<&Script>) -> Heat {
        let written = script.map(|script| script.path);
        let elsewhere = |symbol: &SymbolInfo| is_method(symbol) && Some(symbol.file.as_str()) != written;
        let mut methods: Vec<String> = Vec::new();
        let mut nodes: HashMap<String, usize> = HashMap::new();
        let named = index.symbols.iter().filter(|symbol| elsewhere(symbol)).map(|symbol| &symbol.name).chain(script.into_iter().flat_map(|script| script.methods().map(|symbol| &symbol.name)));
        for name in named {
            nodes.entry(name.clone()).or_insert_with(|| {
                methods.push(name.clone());
                methods.len() - 1
            });
        }
        let mut owners: HashMap<&str, Vec<usize>> = HashMap::new();
        for (at, method) in methods.iter().enumerate() {
            owners.entry(simple(method)).or_default().push(at);
        }
        let mut edges: Vec<Vec<(usize, bool)>> = vec![Vec::new(); methods.len()];
        for (at, callee) in index.symbols.iter().enumerate().filter(|(_, symbol)| is_method(symbol)) {
            let Some(&to) = nodes.get(&callee.name) else { continue };
            let resolved = owners[simple(&callee.name)].len() == 1;
            for &(_, _, from) in index.raw_references(at).iter().filter(|(_, _, from)| *from != NOBODY) {
                let caller = &index.symbols[from as usize];
                if elsewhere(caller) {
                    edges[nodes[&caller.name]].push((to, resolved));
                }
            }
        }
        if let Some(script) = script {
            script_calls(script, &nodes, &owners, &mut edges);
        }
        let components = script.map_or(&index.unity, |script| &script.components);
        let coroutines = coroutines_every_frame(index, script);
        let roots: Vec<usize> = methods.iter().enumerate().filter(|(_, method)| runs_every_frame(components, method) || coroutines.contains(*method)).map(|(at, _)| at).collect();
        let resolved = spread(&edges, &roots, true);
        let any = spread(&edges, &roots, false);
        let reach = (0..methods.len())
            .map(|at| match (resolved[at], any[at]) {
                (Some((0, _)), _) => Reach::Frame,
                (Some((count, root)), _) if count <= NEAR => Reach::Calls { count, from: methods[root].clone() },
                (_, Some((_, root))) => Reach::Far { from: methods[root].clone() },
                _ => Reach::Cold,
            })
            .collect();
        Heat { nodes, reach }
    }

    pub fn reach(&self, method: &str) -> Reach {
        self.nodes.get(method).map_or(Reach::Cold, |&at| self.reach[at].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::fixture;

    const PLAYER: (&str, &str) = (
        "Assets/Scripts/Player.cs",
        "using System.Collections;
using UnityEngine;

public class Player : MonoBehaviour
{
    void Update() { Steer(); }
    void Steer() { Aim(); }
    void Aim() { Track(); Steer(); }
    void Track() { Settle(); }
    void Settle() { }
    void LateUpdate() { Shake(); }
    void Shake() { }
    void Start() { Prepare(); StartCoroutine(Blink()); StartCoroutine(Fade()); }
    void Prepare() { }
    IEnumerator Blink() { while (enabled) { Flash(); yield return null; } }
    void Flash() { }
    IEnumerator Fade() { yield return null; Dim(); }
    void Dim() { }
}
",
    );
    const RIG: (&str, &str) = ("Assets/Scripts/Rig.cs", "using UnityEngine;\n\npublic class Rig : MonoBehaviour\n{\n    public void Shake() { }\n}\n");
    const PLAIN: (&str, &str) = ("Assets/Scripts/Plain.cs", "public class Plain\n{\n    void Update() { Helper(); }\n    void Helper() { }\n}\n");

    fn calls(count: u32, from: &str) -> Reach {
        Reach::Calls { count, from: from.to_string() }
    }

    fn far(from: &str) -> Reach {
        Reach::Far { from: from.to_string() }
    }

    #[test]
    fn what_a_frame_reaches_is_measured_in_calls_and_a_cycle_ends() {
        let heat = Heat::of(&fixture::indexed("sens-canon-hot", "reach", &[PLAYER, RIG, PLAIN]), None);
        let seen = |method: &str| heat.reach(method);
        assert_eq!(seen("Player.Update"), Reach::Frame);
        assert_eq!(seen("Player.Steer"), calls(1, "Player.Update"));
        assert_eq!(seen("Player.Aim"), calls(2, "Player.Update"));
        assert_eq!(seen("Player.Track"), calls(3, "Player.Update"));
        assert_eq!(seen("Player.Settle"), far("Player.Update"));
        assert_eq!((seen("Player.Shake"), seen("Rig.Shake")), (far("Player.LateUpdate"), far("Player.LateUpdate")));
        for cold in ["Player.Start", "Player.Prepare", "Plain.Update", "Plain.Helper", "Nobody.Knows"] {
            assert_eq!(seen(cold), Reach::Cold, "{cold}");
        }
    }

    #[test]
    fn a_coroutine_that_yields_every_frame_inside_a_loop_runs_every_frame() {
        let heat = Heat::of(&fixture::indexed("sens-canon-hot", "coroutines", &[PLAYER]), None);
        assert_eq!(heat.reach("Player.Blink"), Reach::Frame);
        assert_eq!(heat.reach("Player.Flash"), calls(1, "Player.Blink"));
        assert_eq!((heat.reach("Player.Fade"), heat.reach("Player.Dim")), (Reach::Cold, Reach::Cold));
    }

    #[test]
    fn the_file_being_written_brings_its_own_calls_and_its_new_components() {
        let index = fixture::indexed("sens-canon-hot", "written", &[PLAYER, RIG]);
        let rewritten = "public class Player : MonoBehaviour\n{\n    void Update() { Glide(); }\n    void Glide() { }\n    void Track() { }\n}\npublic class Boss : Rig\n{\n    void LateUpdate() { Roar(); }\n    void Roar() { }\n}\n";
        let script = Script::read(&index, "Assets/Scripts/Player.cs", rewritten).unwrap();
        let heat = Heat::of(&index, Some(&script));
        assert_eq!(heat.reach("Player.Glide"), calls(1, "Player.Update"));
        assert_eq!(heat.reach("Player.Track"), Reach::Cold);
        assert_eq!(heat.reach("Boss.Roar"), calls(1, "Boss.LateUpdate"));
        assert_eq!(script.method_at(rewritten.find("Glide();").unwrap()), Some("Player.Update"));
        assert_eq!(script.method_at(0), None);
    }

    #[test]
    fn the_distance_to_a_frame_sets_how_hard_a_rule_speaks_and_cold_code_says_nothing() {
        let graded = |reach: Reach| reach.graded(Severity::Block);
        assert_eq!(graded(Reach::Frame), Some(Severity::Block));
        assert_eq!(graded(calls(2, "Player.Update")), Some(Severity::Consider));
        assert_eq!(graded(far("Player.Update")), Some(Severity::Note));
        assert_eq!(graded(Reach::Cold), None);
        assert_eq!(calls(1, "Player.Update").graded(Severity::Consider), Some(Severity::Note));
        assert_eq!(Reach::Frame.graded(Severity::Note), Some(Severity::Note));
    }
}
