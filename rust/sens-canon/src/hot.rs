use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use sens_index::build;
use sens_index::index::{Index, NOBODY, SymbolInfo};
use sens_index::lang::treesitter::text;
use tree_sitter::Node;

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

fn runs_every_frame(index: &Index, method: &str) -> bool {
    method.rsplit_once('.').is_some_and(|(owner, name)| index.unity.contains_key(owner) && PER_FRAME.contains(&name))
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

fn coroutines_every_frame(index: &Index) -> HashSet<String> {
    let files: BTreeSet<&str> = index.symbols.iter().filter(|symbol| is_method(symbol) && symbol.signature.contains(COROUTINE)).map(|symbol| symbol.file.as_str()).collect();
    let mut found = HashSet::new();
    for file in files {
        let source = std::fs::read_to_string(index.root.join(file)).unwrap_or_default();
        let Some((tree, emitted)) = build::parse(file, &source) else { continue };
        for symbol in emitted.symbols.into_iter().filter(|symbol| symbol.kind == "method" && symbol.signature.contains(COROUTINE)) {
            if tree.root_node().descendant_for_byte_range(symbol.start, symbol.end).is_some_and(|method| yields_every_frame(method, &source, false)) {
                found.insert(symbol.name);
            }
        }
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

impl Heat {
    pub fn of(index: &Index) -> Heat {
        let mut methods: Vec<String> = Vec::new();
        let mut nodes: HashMap<String, usize> = HashMap::new();
        for symbol in index.symbols.iter().filter(|symbol| is_method(symbol)) {
            nodes.entry(symbol.name.clone()).or_insert_with(|| {
                methods.push(symbol.name.clone());
                methods.len() - 1
            });
        }
        let mut owners: HashMap<&str, usize> = HashMap::new();
        for method in &methods {
            *owners.entry(simple(method)).or_default() += 1;
        }
        let mut edges: Vec<Vec<(usize, bool)>> = vec![Vec::new(); methods.len()];
        for (at, callee) in index.symbols.iter().enumerate().filter(|(_, symbol)| is_method(symbol)) {
            let to = nodes[&callee.name];
            let resolved = owners[simple(&callee.name)] == 1;
            for &(_, _, from) in index.raw_references(at).iter().filter(|(_, _, from)| *from != NOBODY) {
                let caller = &index.symbols[from as usize];
                if is_method(caller) {
                    edges[nodes[&caller.name]].push((to, resolved));
                }
            }
        }
        let coroutines = coroutines_every_frame(index);
        let roots: Vec<usize> = methods.iter().enumerate().filter(|(_, method)| runs_every_frame(index, method) || coroutines.contains(*method)).map(|(at, _)| at).collect();
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
        let heat = Heat::of(&fixture::indexed("sens-canon-hot", "reach", &[PLAYER, RIG, PLAIN]));
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
        let heat = Heat::of(&fixture::indexed("sens-canon-hot", "coroutines", &[PLAYER]));
        assert_eq!(heat.reach("Player.Blink"), Reach::Frame);
        assert_eq!(heat.reach("Player.Flash"), calls(1, "Player.Blink"));
        assert_eq!((heat.reach("Player.Fade"), heat.reach("Player.Dim")), (Reach::Cold, Reach::Cold));
    }
}
