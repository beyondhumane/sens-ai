use std::collections::{HashMap, HashSet};

use sens_index::index::Index;
use sens_index::lang::treesitter::text;
use sens_index::unity::MESSAGES;
use tree_sitter::Node;

use crate::hot::{Heat, Reach, Script};
use crate::verdict::{Change, Finding, ProjectRules, Rule, Severity, Target};

const UNMEASURED: Severity = Severity::Note;
const KEY_CAP: usize = 80;
const AWAITABLE_SINCE: (u32, u32) = (2023, 1);
const POOLS_SINCE: (u32, u32) = (2021, 1);
const SET_ASIDE: [&str; 2] = ["editor", "tests"];
const LOOKUPS: [&str; 6] = ["GetComponent", "GetComponents", "GetComponentInChildren", "GetComponentsInChildren", "GetComponentInParent", "GetComponentsInParent"];
const SCENE_SEARCHES: [&str; 5] = ["FindObjectOfType", "FindObjectsOfType", "FindObjectsByType", "FindFirstObjectByType", "FindAnyObjectByType"];
const GAME_OBJECT_SEARCHES: [&str; 4] = ["Find", "FindWithTag", "FindGameObjectWithTag", "FindGameObjectsWithTag"];
const LINQ: [&str; 38] = [
    "Where", "Select", "SelectMany", "OrderBy", "OrderByDescending", "ThenBy", "ThenByDescending", "GroupBy", "ToList", "ToArray", "ToDictionary", "ToHashSet", "ToLookup", "Any", "All", "Count", "First", "FirstOrDefault", "Last",
    "LastOrDefault", "Single", "SingleOrDefault", "Sum", "Average", "Distinct", "Skip", "Take", "SkipWhile", "TakeWhile", "Aggregate", "Zip", "Cast", "OfType", "Except", "Intersect", "Union", "ElementAt", "SequenceEqual",
];
const COLLECTIONS: [&str; 9] = ["List", "Dictionary", "HashSet", "Queue", "Stack", "LinkedList", "SortedList", "SortedDictionary", "SortedSet"];
const ENGINE_ALLOCATIONS: [&str; 7] = ["WaitForSeconds", "WaitForSecondsRealtime", "Texture2D", "Material", "GUIStyle", "Mesh", "RenderTexture"];
const PHYSICS_QUERIES: [&str; 7] = ["RaycastAll", "SphereCastAll", "CapsuleCastAll", "BoxCastAll", "OverlapSphere", "OverlapBox", "OverlapCapsule"];
const PHYSICS_2D_QUERIES: [&str; 9] = ["RaycastAll", "LinecastAll", "CircleCastAll", "BoxCastAll", "CapsuleCastAll", "OverlapCircleAll", "OverlapBoxAll", "OverlapAreaAll", "OverlapPointAll"];
const MESH_ARRAYS: [(&str, &str); 6] = [("vertices", "GetVertices"), ("triangles", "GetTriangles"), ("normals", "GetNormals"), ("tangents", "GetTangents"), ("uv", "GetUVs"), ("colors", "GetColors")];
const BY_NAME: [&str; 27] = [
    "SetFloat", "SetInteger", "SetBool", "SetTrigger", "ResetTrigger", "GetFloat", "GetInteger", "GetBool", "SetInt", "GetInt", "SetColor", "GetColor", "SetVector", "GetVector", "SetTexture", "GetTexture", "SetMatrix",
    "SetTextureOffset", "SetTextureScale", "HasProperty", "SetGlobalFloat", "SetGlobalInt", "SetGlobalInteger", "SetGlobalColor", "SetGlobalVector", "SetGlobalTexture", "SetGlobalMatrix",
];
const KEYED_BY_STRING: [&str; 2] = ["PlayerPrefs", "EditorPrefs"];
const HASHERS: [&str; 2] = ["StringToHash", "PropertyToID"];
const STRINGS: [&str; 4] = ["string_literal", "verbatim_string_literal", "raw_string_literal", "interpolated_string_expression"];
const KEPT_FOR_OVERRIDES: [&str; 3] = ["virtual", "override", "abstract"];
const RESOURCE_LOADS: [&str; 3] = ["Load", "LoadAsync", "LoadAll"];
const SPAWNS: [&str; 2] = ["Instantiate", "Destroy"];
const UNITY_OBJECTS: [&str; 3] = ["Object", "GameObject", "UnityEngine.Object"];
const BY_MESSAGE: [&str; 3] = ["SendMessage", "SendMessageUpwards", "BroadcastMessage"];
const LOGS: [&str; 4] = ["Log", "LogWarning", "LogFormat", "LogWarningFormat"];
const LOGGERS: [&str; 2] = ["Debug", "UnityEngine.Debug"];
const BUILD_GUARDS: [&str; 3] = ["UNITY_EDITOR", "DEVELOPMENT_BUILD", "DEBUG"];
const SERIALIZED_HOLDERS: [&str; 2] = ["MonoBehaviour", "ScriptableObject"];
const NOT_SERIALIZED: [&str; 3] = ["static", "const", "readonly"];

struct Spot {
    rule: Rule,
    line: u32,
    method: Option<String>,
    said: String,
    what: String,
    fix: String,
    hot: bool,
    target: Option<Target>,
}

impl Spot {
    fn identity(&self) -> (Rule, Option<String>, String) {
        (self.rule, self.method.clone(), self.said.clone())
    }
}

struct Reading<'a, 's> {
    script: &'a Script<'s>,
    linq: bool,
    hashes: HashMap<&'s str, Target>,
    subscribed: HashSet<&'s str>,
    awaitable: bool,
    addressables: bool,
    pool: Option<Target>,
    pools: bool,
}

pub fn ships(path: &str) -> bool {
    !path.split('/').any(|folder| SET_ASIDE.contains(&folder.to_lowercase().as_str()))
}

fn squeezed(said: &str) -> String {
    said.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

fn descendants<'t>(node: Node<'t>, out: &mut Vec<Node<'t>>) {
    out.push(node);
    for child in children(node) {
        descendants(child, out);
    }
}

fn name_of<'s>(node: Node, source: &'s str) -> &'s str {
    match node.kind() {
        "generic_name" => node.named_child(0).map_or("", |name| text(&name, source)),
        "qualified_name" => node.child_by_field_name("name").map_or("", |name| name_of(name, source)),
        _ => text(&node, source),
    }
}

fn called<'t, 's>(node: Node<'t>, source: &'s str) -> Option<(&'s str, Option<&'s str>)> {
    if node.kind() != "invocation_expression" {
        return None;
    }
    let function = node.child_by_field_name("function")?;
    match function.kind() {
        "identifier" | "generic_name" => Some((name_of(function, source), None)),
        "member_access_expression" => Some((name_of(function.child_by_field_name("name")?, source), function.child_by_field_name("expression").map(|receiver| text(&receiver, source)))),
        "conditional_access_expression" => {
            let binding = children(function).into_iter().find(|child| child.kind() == "member_binding_expression")?;
            Some((name_of(binding.child_by_field_name("name")?, source), function.child_by_field_name("condition").map(|receiver| text(&receiver, source))))
        }
        _ => None,
    }
}

fn first_argument(node: Node) -> Option<Node> {
    children(node.child_by_field_name("arguments")?.named_child(0)?).pop()
}

fn literal<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    (node.kind() == "string_literal").then(|| node.named_child(0).filter(|content| content.kind() == "string_literal_content").map_or("", |content| text(&content, source)))
}

fn assigned_to(node: Node) -> Option<(Node, Node)> {
    let parent = node.parent().filter(|parent| parent.kind() == "assignment_expression")?;
    let operator = parent.child_by_field_name("operator")?;
    let left = parent.child_by_field_name("left")?;
    (parent.child_by_field_name("right") == Some(node)).then_some((left, operator))
}

fn stored_lazily(node: Node, source: &str) -> bool {
    let Some((left, operator)) = assigned_to(node) else { return false };
    if text(&operator, source) == "??=" {
        return true;
    }
    let field = text(&left, source);
    let mut above = node.parent();
    while let Some(at) = above.filter(|at| at.kind() != "method_declaration") {
        if at.kind() == "if_statement" && at.child_by_field_name("condition").is_some_and(|condition| text(&condition, source).contains(field)) {
            return true;
        }
        above = at.parent();
    }
    false
}

fn is_written(node: Node) -> bool {
    node.parent().is_some_and(|parent| parent.kind() == "assignment_expression" && parent.child_by_field_name("left") == Some(node))
}

impl<'a, 's> Reading<'a, 's> {
    fn of(script: &'a Script<'s>, index: &Index, rules: &ProjectRules) -> Reading<'a, 's> {
        let source = script.source;
        let mut all = Vec::new();
        descendants(script.tree.root_node(), &mut all);
        let linq = all.iter().any(|node| node.kind() == "using_directive" && text(node, source).contains("System.Linq"));
        let mut hashes = HashMap::new();
        let mut subscribed = HashSet::new();
        for node in &all {
            if let Some((name, _)) = called(*node, source).filter(|(name, _)| HASHERS.contains(name))
                && let Some(said) = first_argument(*node).and_then(|argument| literal(argument, source))
                && let Some(field) = ancestor(*node, "variable_declarator").and_then(|declarator| declarator.child_by_field_name("name"))
            {
                let line = field.start_position().row as u32 + 1;
                let declared = ancestor(*node, "field_declaration").map_or_else(|| squeezed(text(node, source)), |declaration| squeezed(text(&declaration, source)));
                hashes.entry(said).or_insert(Target { symbol: text(&field, source).to_string(), file: script.path.to_string(), line, signature: declared.clone(), excerpt: format!("{name}(\"{said}\")") });
            }
            if node.kind() == "assignment_expression" && node.child_by_field_name("operator").is_some_and(|operator| text(&operator, source) == "+=") {
                subscribed.extend(node.child_by_field_name("right").filter(|right| right.kind() == "identifier").map(|right| text(&right, source)));
            }
            if node.kind() == "argument" {
                subscribed.extend(node.named_child(0).filter(|value| value.kind() == "identifier").map(|value| text(&value, source)));
            }
        }
        let pool = index.symbols.iter().find(|symbol| symbol.kind == "class" && symbol.file.ends_with(".cs") && symbol.name.ends_with("Pool")).map(|symbol| Target {
            symbol: symbol.name.clone(),
            file: symbol.file.clone(),
            line: symbol.line,
            signature: symbol.signature.clone(),
            excerpt: symbol.signature.clone(),
        });
        let editor = rules.editor.as_deref();
        Reading { script, linq, hashes, subscribed, awaitable: since(editor, AWAITABLE_SINCE), addressables: rules.addressables, pool, pools: since(editor, POOLS_SINCE) }
    }

    fn spot(&self, rule: Rule, node: Node, what: String, fix: String) -> Spot {
        let source = self.script.source;
        Spot { rule, line: node.start_position().row as u32 + 1, method: self.script.method_at(node.start_byte()).map(str::to_string), said: squeezed(text(&node, source)), what, fix, hot: true, target: None }
    }

    fn message(&self, rule: Rule, node: Node, what: String, fix: String) -> Spot {
        Spot { hot: false, ..self.spot(rule, node, what, fix) }
    }
}

fn ancestor<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut above = node.parent();
    while let Some(at) = above {
        if at.kind() == kind {
            return Some(at);
        }
        above = at.parent();
    }
    None
}

fn since(editor: Option<&str>, release: (u32, u32)) -> bool {
    let mut parts = editor.unwrap_or_default().split('.').map(|part| part.parse::<u32>().unwrap_or_default());
    (parts.next().unwrap_or_default(), parts.next().unwrap_or_default()) >= release
}

fn lookup(node: Node, reading: &Reading) -> Option<Spot> {
    let (name, _) = called(node, reading.script.source)?;
    (LOOKUPS.contains(&name) && !stored_lazily(node, reading.script.source)).then(|| reading.spot(Rule::U1, node, format!("`{name}` searches the components on every call"), "Look it up once in `Awake` and keep it in a field.".into()))
}

fn main_camera(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    let reads = node.kind() == "member_access_expression"
        && node.child_by_field_name("expression").is_some_and(|receiver| text(&receiver, source) == "Camera")
        && node.child_by_field_name("name").is_some_and(|name| text(&name, source) == "main");
    (reads && !stored_lazily(node, source)).then(|| reading.spot(Rule::U2, node, "`Camera.main` looks the camera up on every read".into(), "Keep the camera in a field set in `Awake`.".into()))
}

fn scene_search(node: Node, reading: &Reading) -> Option<Spot> {
    let (name, receiver) = called(node, reading.script.source)?;
    let searches = SCENE_SEARCHES.contains(&name) || (receiver == Some("GameObject") && GAME_OBJECT_SEARCHES.contains(&name));
    (searches && !stored_lazily(node, reading.script.source)).then(|| {
        let said = receiver.map_or_else(|| name.to_string(), |receiver| format!("{receiver}.{name}"));
        reading.spot(Rule::U3, node, format!("`{said}` searches the scene"), "Find it once in `Awake`, or give the field `[SerializeField]` and assign it in the Inspector.".into())
    })
}

fn allocation(node: Node, reading: &Reading, thrown: bool) -> Option<Spot> {
    let source = reading.script.source;
    let (what, fix) = match node.kind() {
        "invocation_expression" => {
            let (name, receiver) = called(node, source)?;
            let linq = reading.linq && receiver.is_some_and(|receiver| receiver != "string" && receiver != "String") && LINQ.contains(&name);
            linq.then(|| (format!("LINQ's `{name}` allocates"), "Write the loop by hand over a list kept in a field.".to_string()))?
        }
        "query_expression" => ("A LINQ query allocates".to_string(), "Write the loop by hand over a list kept in a field.".to_string()),
        "object_creation_expression" => {
            let created = name_of(node.child_by_field_name("type")?, source);
            if COLLECTIONS.contains(&created) {
                (format!("`new {created}` allocates"), "Keep it in a field and `Clear()` it before each use.".to_string())
            } else if ENGINE_ALLOCATIONS.contains(&created) {
                (format!("`new {created}` allocates"), format!("Create the `{created}` once, keep it in a field and reuse it."))
            } else {
                return None;
            }
        }
        "array_creation_expression" | "implicit_array_creation_expression" => ("A new array allocates".to_string(), "Keep the array in a field and reuse it.".to_string()),
        "interpolated_string_expression" if !thrown => ("Building a string allocates".to_string(), "Build it only when the value changes and keep it in a field.".to_string()),
        "binary_expression" if !thrown => {
            let joins = node.child_by_field_name("operator").is_some_and(|operator| text(&operator, source) == "+")
                && [node.child_by_field_name("left"), node.child_by_field_name("right")].into_iter().flatten().any(|side| STRINGS.contains(&side.kind()));
            joins.then(|| ("Joining strings allocates".to_string(), "Build it only when the value changes and keep it in a field.".to_string()))?
        }
        _ => return None,
    };
    Some(reading.spot(Rule::U4, node, what, fix))
}

fn allocating_query(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    if let Some((name, Some(receiver))) = called(node, source) {
        let physics = receiver == "Physics" && PHYSICS_QUERIES.contains(&name);
        let physics_2d = receiver == "Physics2D" && PHYSICS_2D_QUERIES.contains(&name);
        let base = name.trim_end_matches("All");
        let fix = if physics { format!("Use `Physics.{base}NonAlloc` with a buffer kept in a field.") } else { format!("Use the overload of `Physics2D.{base}` that fills a list kept in a field.") };
        return (physics || physics_2d).then(|| reading.spot(Rule::U5, node, format!("`{receiver}.{name}` returns a new array"), fix));
    }
    if node.kind() != "member_access_expression" || is_written(node) {
        return None;
    }
    let receiver = text(&node.child_by_field_name("expression")?, source);
    let read = text(&node.child_by_field_name("name")?, source);
    let (_, getter) = MESH_ARRAYS.iter().find(|(array, _)| *array == read)?;
    receiver.to_lowercase().ends_with("mesh").then(|| reading.spot(Rule::U5, node, format!("Reading `{receiver}.{read}` copies the whole array"), format!("Use `{receiver}.{getter}` with a list kept in a field.")))
}

fn named_property(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    let (name, receiver) = called(node, source)?;
    if !BY_NAME.contains(&name) || receiver.is_some_and(|receiver| KEYED_BY_STRING.contains(&receiver)) {
        return None;
    }
    let said = literal(first_argument(node)?, source)?;
    let what = format!("`{name}(\"{said}\")` turns the name into an id on every call");
    Some(match reading.hashes.get(said) {
        Some(hash) => Spot { target: Some(hash.clone()), ..reading.spot(Rule::U6, node, what, format!("Use `{}`, declared at line {}.", hash.symbol, hash.line)) },
        None => reading.spot(Rule::U6, node, what, format!("Keep `Animator.StringToHash(\"{said}\")`, or `Shader.PropertyToID(\"{said}\")` for a material, in a `static readonly int` field.")),
    })
}

fn modifiers<'s>(node: Node, source: &'s str) -> Vec<&'s str> {
    children(node).into_iter().filter(|child| child.kind() == "modifier").map(|child| text(&child, source)).collect()
}

fn unity_message(node: Node, reading: &Reading) -> Option<String> {
    let simple = text(&node.child_by_field_name("name")?, reading.script.source);
    let method = reading.script.method_at(node.start_byte())?;
    let owner = method.rsplit_once('.').map_or("", |(owner, _)| owner);
    (MESSAGES.contains(&simple) && reading.script.components.contains_key(owner)).then(|| method.to_string())
}

fn empty_message(node: Node, reading: &Reading) -> Option<Spot> {
    if node.kind() != "method_declaration" || modifiers(node, reading.script.source).iter().any(|modifier| KEPT_FOR_OVERRIDES.contains(modifier)) {
        return None;
    }
    let body = node.child_by_field_name("body").filter(|body| body.kind() == "block")?;
    let method = unity_message(node, reading)?;
    children(body).iter().all(|child| child.kind() == "comment").then(|| reading.message(Rule::U7, node, format!("`{method}` is an empty Unity message"), "Delete it: Unity calls it anyway.".into()))
}

fn handles_an_event(node: Node, source: &str) -> bool {
    let parameters = node.child_by_field_name("parameters").map(children).unwrap_or_default();
    parameters.len() == 2 && parameters[1].child_by_field_name("type").is_some_and(|kind| text(&kind, source).ends_with("EventArgs"))
}

fn async_void(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    if node.kind() != "method_declaration" || node.child_by_field_name("returns").is_none_or(|returns| text(&returns, source) != "void") {
        return None;
    }
    let shown = modifiers(node, source);
    let simple = text(&node.child_by_field_name("name")?, source);
    let method = reading.script.method_at(node.start_byte())?;
    let owner = method.rsplit_once('.').map_or("", |(owner, _)| owner);
    let exempt = !shown.contains(&"async")
        || shown.contains(&"public")
        || !reading.script.components.contains_key(owner)
        || unity_message(node, reading).is_some()
        || reading.subscribed.contains(simple)
        || handles_an_event(node, source);
    if exempt {
        return None;
    }
    let fix = if reading.awaitable { "Return `Awaitable`, or `Task`, so the caller can await it." } else { "Return `Task` so the caller can await it." };
    Some(reading.message(Rule::U8, node, format!("`async void {simple}` cannot be awaited, and an exception in it reaches no caller"), fix.into()))
}

fn resources_load(node: Node, reading: &Reading) -> Option<Spot> {
    let (name, receiver) = called(node, reading.script.source)?;
    (reading.addressables && receiver == Some("Resources") && RESOURCE_LOADS.contains(&name)).then(|| {
        reading.message(Rule::U9, node, format!("`Resources.{name}` reads from the Resources folders, which ship whole in every build"), "Load it with `Addressables.LoadAssetAsync`: the project already has Addressables.".into())
    })
}

fn spawn(node: Node, reading: &Reading) -> Option<Spot> {
    let (name, receiver) = called(node, reading.script.source)?;
    if !SPAWNS.contains(&name) || receiver.is_some_and(|receiver| !UNITY_OBJECTS.contains(&receiver)) {
        return None;
    }
    let what = format!("`{name}` creates or destroys a whole object");
    let back = "and give it back instead of destroying it.";
    Some(match &reading.pool {
        Some(pool) => Spot { target: Some(pool.clone()), ..reading.spot(Rule::U10, node, what, format!("Take it from `{}`, which the project already has, {back}", pool.symbol)) },
        None if reading.pools => reading.spot(Rule::U10, node, what, format!("Take it from a `UnityEngine.Pool.ObjectPool<T>` {back}")),
        None => reading.spot(Rule::U10, node, what, format!("Take it from a pool {back}")),
    })
}

fn by_message(node: Node, reading: &Reading) -> Option<Spot> {
    let (name, _) = called(node, reading.script.source)?;
    BY_MESSAGE.contains(&name).then(|| reading.message(Rule::U11, node, format!("`{name}` looks the method up by its name while the game runs"), "Call it through a direct reference or an event.".into()))
}

fn is_tag(node: Node, source: &str) -> bool {
    match node.kind() {
        "identifier" => text(&node, source) == "tag",
        "member_access_expression" => node.child_by_field_name("name").is_some_and(|name| text(&name, source) == "tag"),
        _ => false,
    }
}

fn tag_compared(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    if node.kind() != "binary_expression" {
        return None;
    }
    let operator = text(&node.child_by_field_name("operator")?, source);
    let (left, right) = (node.child_by_field_name("left")?, node.child_by_field_name("right")?);
    let (tag, other) = if is_tag(left, source) { (left, right) } else { (right, left) };
    if !matches!(operator, "==" | "!=") || !is_tag(tag, source) {
        return None;
    }
    let owner = tag.child_by_field_name("expression").map_or_else(String::new, |owner| format!("{}.", text(&owner, source)));
    let negated = if operator == "!=" { "!" } else { "" };
    let compared = text(&other, source);
    Some(reading.message(Rule::U12, node, format!("Reading `{}` builds a new string to compare it", text(&tag, source)), format!("Use `{negated}{owner}CompareTag({compared})`, which compares without allocating.")))
}

fn attribute_names<'s>(node: Node, source: &'s str) -> Vec<&'s str> {
    children(node)
        .into_iter()
        .filter(|child| child.kind() == "attribute_list")
        .flat_map(children)
        .filter_map(|attribute| attribute.child_by_field_name("name"))
        .map(|name| {
            let said = name_of(name, source);
            said.strip_suffix("Attribute").unwrap_or(said)
        })
        .collect()
}

fn owner_root(node: Node, reading: &Reading) -> Option<&'static str> {
    let class = ancestor(node, "class_declaration")?;
    reading.script.components.get(text(&class.child_by_field_name("name")?, reading.script.source)).copied()
}

fn public_field(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    if node.kind() != "field_declaration" {
        return None;
    }
    let shown = modifiers(node, source);
    let serialized_in_public = shown.contains(&"public") && !shown.iter().any(|modifier| NOT_SERIALIZED.contains(modifier)) && !attribute_names(node, source).contains(&"NonSerialized");
    if !serialized_in_public || !owner_root(node, reading).is_some_and(|root| SERIALIZED_HOLDERS.contains(&root)) {
        return None;
    }
    let names: Vec<&str> = children(node)
        .into_iter()
        .filter(|child| child.kind() == "variable_declaration")
        .flat_map(children)
        .filter(|child| child.kind() == "variable_declarator")
        .filter_map(|declarator| declarator.child_by_field_name("name"))
        .map(|name| text(&name, source))
        .collect();
    Some(reading.message(
        Rule::U13,
        node,
        format!("`{}` is public so the Inspector shows it, which also lets any script change it", names.join("`, `")),
        "Make it `[SerializeField] private` if no other script needs to write it: the Inspector still shows it.".into(),
    ))
}

fn guarded(node: Node, source: &str) -> bool {
    let (mut below, mut above) = (node, node.parent());
    while let Some(at) = above {
        let condition = at.child_by_field_name("condition").map(|condition| text(&condition, source)).unwrap_or_default();
        let clear = !condition.contains('!');
        let kept = match at.kind() {
            "preproc_if" | "preproc_elif" => at.child_by_field_name("alternative") != Some(below) && clear && BUILD_GUARDS.iter().any(|guard| condition.contains(guard)),
            "if_statement" => at.child_by_field_name("consequence") == Some(below) && clear && condition.contains("isDebugBuild"),
            "method_declaration" => return attribute_names(at, source).contains(&"Conditional"),
            _ => false,
        };
        if kept {
            return true;
        }
        (below, above) = (at, at.parent());
    }
    false
}

fn shipped_log(node: Node, reading: &Reading) -> Option<Spot> {
    let source = reading.script.source;
    let (name, receiver) = called(node, source)?;
    let logs = LOGS.contains(&name) && receiver.is_some_and(|receiver| LOGGERS.contains(&receiver));
    (logs && !guarded(node, source)).then(|| {
        reading.spot(Rule::U14, node, format!("`Debug.{name}` builds its text and its stack trace in the build too"), "Wrap it in `#if UNITY_EDITOR || DEVELOPMENT_BUILD`, or log through a method marked `[Conditional(\"UNITY_EDITOR\")]`.".into())
    })
}

fn immediate_gui(node: Node, reading: &Reading) -> Option<Spot> {
    let named_on_gui = node.kind() == "method_declaration" && node.child_by_field_name("name").is_some_and(|name| text(&name, reading.script.source) == "OnGUI");
    if !named_on_gui || owner_root(node, reading) != Some("MonoBehaviour") {
        return None;
    }
    let method = reading.script.method_at(node.start_byte())?;
    Some(reading.message(Rule::U15, node, format!("`{method}` draws with IMGUI, which runs several times per frame and allocates"), "Draw the interface with UI Toolkit or uGUI.".into()))
}

fn spots(node: Node, reading: &Reading, thrown: bool, out: &mut Vec<Spot>) {
    let thrown = thrown || matches!(node.kind(), "throw_statement" | "throw_expression");
    let found = [
        lookup(node, reading),
        main_camera(node, reading),
        scene_search(node, reading),
        allocation(node, reading, thrown),
        allocating_query(node, reading),
        named_property(node, reading),
        empty_message(node, reading),
        async_void(node, reading),
        resources_load(node, reading),
        spawn(node, reading),
        by_message(node, reading),
        tag_compared(node, reading),
        public_field(node, reading),
        shipped_log(node, reading),
        immediate_gui(node, reading),
    ];
    out.extend(found.into_iter().flatten());
    for child in children(node) {
        spots(child, reading, thrown, out);
    }
}

fn found_in(script: &Script, index: &Index, rules: &ProjectRules) -> Vec<Spot> {
    let reading = Reading::of(script, index, rules);
    let mut found = Vec::new();
    spots(script.tree.root_node(), &reading, false, &mut found);
    found
}

fn place(method: &str, reach: &Reach) -> String {
    match reach {
        Reach::Frame => format!("in `{method}`, which runs every frame"),
        Reach::Calls { count, from } => format!("in `{method}`, {count} call{} from `{from}`, which runs every frame", if *count == 1 { "" } else { "s" }),
        Reach::Far { from } => format!("in `{method}`, which `{from}` may reach every frame through more than 3 calls or a call Sens cannot tell apart"),
        Reach::Cold => format!("in `{method}`"),
    }
}

fn finding(spot: Spot, heat: Option<&Heat>, path: &str) -> Option<Finding> {
    let method = spot.method.clone().unwrap_or_default();
    let (severity, what) = if spot.hot {
        let reach = heat?.reach(spot.method.as_deref()?);
        (reach.graded(UNMEASURED)?, format!("{} {}", spot.what, place(&method, &reach)))
    } else {
        (UNMEASURED, spot.what)
    };
    Some(Finding {
        rule: spot.rule,
        severity,
        file: path.to_string(),
        line: spot.line,
        message: format!("{what}. {}", spot.fix),
        target: spot.target,
        key: format!("{:?}:{path}:{method}:{}", spot.rule, spot.said.chars().take(KEY_CAP).collect::<String>()),
    })
}

pub fn findings(index: &Index, change: &Change, rules: &ProjectRules) -> Vec<Finding> {
    let Some(after) = change.after.as_deref().filter(|_| change.path.ends_with(".cs") && ships(&change.path)) else {
        return Vec::new();
    };
    let Some(script) = Script::read(index, &change.path, after) else {
        return Vec::new();
    };
    let mut known: HashMap<(Rule, Option<String>, String), usize> = HashMap::new();
    if let Some(before) = change.before.as_deref().and_then(|before| Script::read(index, &change.path, before)) {
        for spot in found_in(&before, index, rules) {
            *known.entry(spot.identity()).or_default() += 1;
        }
    }
    let mut lines: HashSet<(Rule, u32)> = HashSet::new();
    let mut fresh: Vec<Spot> = found_in(&script, index, rules)
        .into_iter()
        .filter(|spot| match known.get_mut(&spot.identity()) {
            Some(left) if *left > 0 => {
                *left -= 1;
                false
            }
            _ => true,
        })
        .filter(|spot| lines.insert((spot.rule, spot.line)))
        .collect();
    let allocating: HashSet<u32> = fresh.iter().filter(|spot| spot.rule == Rule::U4).map(|spot| spot.line).collect();
    fresh.retain(|spot| spot.rule != Rule::U14 || !allocating.contains(&spot.line));
    let heat = fresh.iter().any(|spot| spot.hot).then(|| Heat::of(index, Some(&script)));
    fresh.into_iter().filter_map(|spot| finding(spot, heat.as_ref(), &change.path)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: &str = "Assets/Scripts/Runner.cs";

    const CLEAN: &str = "using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

public class Runner : MonoBehaviour
{
    static readonly int Speed = Animator.StringToHash(\"Speed\");
    readonly List<Collider> touching = new List<Collider>();
    readonly RaycastHit[] hits = new RaycastHit[8];
    readonly WaitForSeconds pause = new WaitForSeconds(0.5f);
    Rigidbody body;
    Animator animator;
    Camera view;
    string label;

    void Awake()
    {
        body = GetComponent<Rigidbody>();
        animator = GetComponent<Animator>();
        label = $\"Runner {name}\";
    }

    void Update()
    {
        if (view == null)
        {
            view = Camera.main;
        }
        animator.SetFloat(Speed, body.velocity.magnitude);
        int count = Physics.RaycastNonAlloc(view.transform.position, view.transform.forward, hits);
        touching.Clear();
        for (int i = 0; i < count; i++)
        {
            touching.Add(hits[i].collider);
        }
        if (count > hits.Length) throw new System.InvalidOperationException($\"Too many hits: {count}\");
    }

    IEnumerator Patrol()
    {
        while (enabled)
        {
            yield return pause;
        }
    }

    public async void OnStartPressed()
    {
        await Load();
    }

    async System.Threading.Tasks.Task Load()
    {
        var ready = touching.Where(collider => collider.enabled).ToList();
        await System.Threading.Tasks.Task.Yield();
    }
}
";

    const CARELESS: &str = "using System.Linq;
using UnityEngine;

public class Careless : MonoBehaviour
{
    public Animator animator;
    public Mesh mesh;

    void Start() { }

    void Update()
    {
        var body = GetComponent<Rigidbody>();
        var forward = Camera.main.transform.forward;
        var player = FindObjectOfType<Player>();
        var near = FindObjectsOfType<Enemy>().Where(enemy => enemy.enabled).ToList();
        var hits = Physics.RaycastAll(transform.position, forward);
        animator.SetFloat(\"Speed\", body.velocity.magnitude);
        Steer();
    }

    void Steer()
    {
        var points = mesh.vertices;
        Debug.Log(\"Steering \" + points.Length);
    }

    async void Reload()
    {
        await System.Threading.Tasks.Task.Yield();
    }
}
";

    fn rules(editor: Option<&str>) -> ProjectRules {
        ProjectRules { unity: Some(true), editor: editor.map(str::to_string), ..ProjectRules::default() }
    }

    fn judged(index: &Index, after: &str, rules: &ProjectRules) -> Vec<Finding> {
        findings(index, &Change { path: PATH.into(), before: None, after: Some(after.into()) }, rules)
    }

    fn changed(before: Option<&str>, after: &str, editor: Option<&str>) -> Vec<Finding> {
        findings(&Index::default(), &Change { path: PATH.into(), before: before.map(str::to_string), after: Some(after.into()) }, &rules(editor))
    }

    fn written(after: &str) -> Vec<Finding> {
        changed(None, after, None)
    }

    fn seen(found: &[Finding]) -> Vec<(u32, Rule)> {
        let mut seen: Vec<(u32, Rule)> = found.iter().map(|finding| (finding.line, finding.rule)).collect();
        seen.sort();
        seen
    }

    fn line_of(source: &str, said: &str) -> u32 {
        source.lines().position(|line| line.contains(said)).unwrap_or_else(|| panic!("{said}")) as u32 + 1
    }

    #[test]
    fn a_clean_component_gets_no_finding() {
        let found = written(CLEAN);
        assert!(found.is_empty(), "{found:#?}");
    }

    #[test]
    fn a_careless_component_gets_exactly_its_findings() {
        let found = written(CARELESS);
        let expected = [
            (6, Rule::U13),
            (7, Rule::U13),
            (9, Rule::U7),
            (13, Rule::U1),
            (14, Rule::U2),
            (15, Rule::U3),
            (16, Rule::U3),
            (16, Rule::U4),
            (17, Rule::U5),
            (18, Rule::U6),
            (24, Rule::U5),
            (25, Rule::U4),
            (28, Rule::U8),
        ];
        assert_eq!(seen(&found), expected, "{found:#?}");
        assert!(found.iter().all(|finding| finding.severity == Severity::Note), "{found:#?}");
        let said = |line: u32, rule: Rule| found.iter().find(|finding| finding.line == line && finding.rule == rule).map(|finding| finding.message.as_str()).unwrap_or_default();
        assert_eq!(said(13, Rule::U1), "`GetComponent` searches the components on every call in `Careless.Update`, which runs every frame. Look it up once in `Awake` and keep it in a field.");
        assert_eq!(said(17, Rule::U5), "`Physics.RaycastAll` returns a new array in `Careless.Update`, which runs every frame. Use `Physics.RaycastNonAlloc` with a buffer kept in a field.");
        assert!(said(24, Rule::U5).contains("in `Careless.Steer`, 1 call from `Careless.Update`") && said(24, Rule::U5).contains("`mesh.GetVertices`"), "{}", said(24, Rule::U5));
        assert_eq!(said(9, Rule::U7), "`Careless.Start` is an empty Unity message. Delete it: Unity calls it anyway.");
        assert!(said(28, Rule::U8).ends_with("Return `Task` so the caller can await it."), "{}", said(28, Rule::U8));
    }

    #[test]
    fn the_same_lookup_speaks_in_update_and_one_call_away_but_not_where_only_start_reaches() {
        let mover = "using UnityEngine;

public class Mover : MonoBehaviour
{
    void Update()
    {
        var a = GetComponent<Rigidbody>();
        Steer();
    }

    void Steer()
    {
        var b = GetComponent<Rigidbody>();
    }

    void Awake()
    {
        Prepare();
    }

    void Prepare()
    {
        var c = GetComponent<Rigidbody>();
    }
}
";
        let found = written(mover);
        let told: Vec<(u32, &str)> = found.iter().map(|finding| (finding.line, finding.message.as_str())).collect();
        assert_eq!(told.len(), 2, "{told:#?}");
        assert!(told[0].0 == 7 && told[0].1.contains("in `Mover.Update`, which runs every frame"), "{told:#?}");
        assert!(told[1].0 == 13 && told[1].1.contains("in `Mover.Steer`, 1 call from `Mover.Update`, which runs every frame"), "{told:#?}");
    }

    #[test]
    fn a_lookup_kept_lazily_in_a_field_is_a_cache_and_any_other_is_not() {
        let lazy = "using UnityEngine;

public class Lazy : MonoBehaviour
{
    Rigidbody body;
    Camera view;
    bool jumping;

    void Update()
    {
        body ??= GetComponent<Rigidbody>();
        if (view == null) view = Camera.main;
        if (jumping) body = GetComponent<Rigidbody>();
        body = GetComponent<Rigidbody>();
    }
}
";
        assert_eq!(seen(&written(lazy)), [(13, Rule::U1), (14, Rule::U1)]);
    }

    #[test]
    fn what_the_file_already_had_is_not_told_again() {
        let before = "public class Mover : MonoBehaviour\n{\n    void Update()\n    {\n        var a = GetComponent<Rigidbody>();\n    }\n}\n";
        let after = "public class Mover : MonoBehaviour\n{\n    void Update()\n    {\n        var a = GetComponent<Rigidbody>();\n        var b = GetComponent<Animator>();\n    }\n}\n";
        assert!(changed(Some(before), before, None).is_empty());
        assert_eq!(seen(&changed(Some(before), after, None)), [(6, Rule::U1)]);
    }

    #[test]
    fn a_property_named_by_string_points_at_the_hash_the_file_already_keeps() {
        let legs = "using UnityEngine;

public class Legs : MonoBehaviour
{
    static readonly int Speed = Animator.StringToHash(\"Speed\");
    [SerializeField] Animator animator;

    void Update()
    {
        animator.SetFloat(\"Speed\", 1f);
        animator.SetBool(\"Grounded\", true);
        PlayerPrefs.SetInt(\"Coins\", 3);
    }
}
";
        let found = written(legs);
        assert_eq!(seen(&found), [(10, Rule::U6), (11, Rule::U6)], "{found:#?}");
        let kept = found[0].target.as_ref().expect("the hash the file keeps");
        assert_eq!((kept.symbol.as_str(), kept.line), ("Speed", 5));
        assert!(found[0].message.ends_with("Use `Speed`, declared at line 5."), "{}", found[0].message);
        assert!(found[1].target.is_none() && found[1].message.contains("Animator.StringToHash(\"Grounded\")"), "{}", found[1].message);
    }

    #[test]
    fn handlers_messages_overrides_and_plain_classes_keep_their_async_void_and_empty_methods() {
        let menu = "using System;
using UnityEngine;
using UnityEngine.UI;

public class Menu : MonoBehaviour
{
    [SerializeField] Button play;
    event EventHandler Saved;

    protected virtual void Update() { }

    void OnEnable()
    {
        play.onClick.AddListener(Play);
        Saved += Store;
    }

    async void Play() { await Awaitable.NextFrameAsync(); }
    async void Store(object sender, EventArgs e) { await Awaitable.NextFrameAsync(); }
    async void Start() { await Awaitable.NextFrameAsync(); }
    async void Quit() { await Awaitable.NextFrameAsync(); }
}

public class Plain
{
    void Update() { }
    async void Later() { }
}
";
        let quit = line_of(menu, "async void Quit");
        assert_eq!(seen(&written(menu)), [(quit, Rule::U8)]);
        let newer = changed(None, menu, Some("2023.2.20f1"));
        assert!(newer[0].message.ends_with("Return `Awaitable`, or `Task`, so the caller can await it."), "{}", newer[0].message);
        assert!(changed(None, menu, Some("6000.0.23f1"))[0].message.contains("`Awaitable`"));
        assert!(!changed(None, menu, Some("2022.3.10f1"))[0].message.contains("`Awaitable`"));
    }

    #[test]
    fn only_csharp_files_are_read() {
        let found = findings(&Index::default(), &Change { path: "Assets/Scripts/notes.txt".into(), before: None, after: Some(CARELESS.into()) }, &rules(None));
        assert!(found.is_empty());
    }

    #[test]
    fn resources_load_speaks_only_when_the_project_has_addressables() {
        let loader = "using UnityEngine;

public class Loader : MonoBehaviour
{
    void Start()
    {
        var level = Resources.Load<GameObject>(\"Levels/One\");
    }
}
";
        assert!(written(loader).is_empty());
        let found = judged(&Index::default(), loader, &ProjectRules { addressables: true, ..rules(None) });
        assert_eq!(seen(&found), [(7, Rule::U9)]);
        assert!(found[0].message.ends_with("Load it with `Addressables.LoadAssetAsync`: the project already has Addressables."), "{}", found[0].message);
    }

    #[test]
    fn a_log_in_update_is_one_finding_and_a_guarded_or_cold_one_is_none() {
        let probe = "using UnityEngine;

public class Probe : MonoBehaviour
{
    int frame;

    void Update()
    {
        Debug.Log($\"frame {frame}\");
        Debug.Log(\"tick\");
        Debug.LogError(\"broken\");
#if UNITY_EDITOR
        Debug.Log(\"editor\");
#else
        Debug.Log(\"player\");
#endif
#if !UNITY_EDITOR
        Debug.LogWarning(\"outside\");
#endif
        if (Debug.isDebugBuild) Debug.Log(\"debug\");
        Trace();
    }

    [System.Diagnostics.Conditional(\"UNITY_EDITOR\")]
    void Trace()
    {
        Debug.Log(\"trace\");
    }

    void Start()
    {
        Debug.Log(\"start\");
    }
}
";
        let at = |said: &str| line_of(probe, said);
        assert_eq!(seen(&written(probe)), [(at("frame {frame}"), Rule::U4), (at("\"tick\""), Rule::U14), (at("\"player\""), Rule::U14), (at("\"outside\""), Rule::U14)]);
    }

    #[test]
    fn spawning_every_frame_points_at_the_pool_the_project_already_has() {
        let gun = "using UnityEngine;

public class Gun : MonoBehaviour
{
    public GameObject bullet;
    Spawner spawner;

    void Update()
    {
        Instantiate(bullet);
        spawner.Destroy(bullet);
    }

    void OnTriggerEnter(Collider other)
    {
        Destroy(other.gameObject);
    }
}
";
        let pooled = sens_index::fixture::indexed("sens-canon-unity", "pool", &[("Assets/Scripts/BulletPool.cs", "public class BulletPool\n{\n}\n")]);
        let found = judged(&pooled, gun, &rules(Some("2022.3.10f1")));
        let spawned: Vec<&Finding> = found.iter().filter(|finding| finding.rule == Rule::U10).collect();
        assert_eq!(spawned.iter().map(|finding| finding.line).collect::<Vec<_>>(), [10], "{found:#?}");
        assert_eq!(spawned[0].target.as_ref().map(|pool| (pool.symbol.as_str(), pool.file.as_str())), Some(("BulletPool", "Assets/Scripts/BulletPool.cs")));
        assert!(spawned[0].message.contains("Take it from `BulletPool`, which the project already has"), "{}", spawned[0].message);
        let fix = |editor: &str| judged(&Index::default(), gun, &rules(Some(editor))).into_iter().find(|finding| finding.rule == Rule::U10).map(|finding| finding.message).unwrap_or_default();
        assert!(fix("2021.3.0f1").contains("`UnityEngine.Pool.ObjectPool<T>`"), "{}", fix("2021.3.0f1"));
        assert!(fix("2020.3.48f1").contains("Take it from a pool"), "{}", fix("2020.3.48f1"));
    }

    #[test]
    fn messages_by_name_and_tags_read_as_strings_have_a_direct_way() {
        let door = "using UnityEngine;

public class Door : MonoBehaviour
{
    void OnTriggerEnter(Collider other)
    {
        if (other.tag == \"Player\") SendMessage(\"Open\");
        if (tag != \"Locked\") other.BroadcastMessage(\"Knock\");
        if (\"Enemy\" == other.tag) Open();
        if (other.CompareTag(\"Player\")) Open();
    }

    void Open()
    {
        transform.Rotate(0f, 90f, 0f);
    }
}
";
        let found = written(door);
        assert_eq!(seen(&found), [(7, Rule::U11), (7, Rule::U12), (8, Rule::U11), (8, Rule::U12), (9, Rule::U12)]);
        let said = |line: u32, rule: Rule| found.iter().find(|finding| finding.line == line && finding.rule == rule).map(|finding| finding.message.clone()).unwrap_or_default();
        assert!(said(7, Rule::U12).ends_with("Use `other.CompareTag(\"Player\")`, which compares without allocating."), "{}", said(7, Rule::U12));
        assert!(said(8, Rule::U12).ends_with("Use `!CompareTag(\"Locked\")`, which compares without allocating."), "{}", said(8, Rule::U12));
        assert!(said(9, Rule::U12).ends_with("Use `other.CompareTag(\"Enemy\")`, which compares without allocating."), "{}", said(9, Rule::U12));
    }

    #[test]
    fn only_a_public_field_unity_serializes_could_be_serialize_field_private() {
        let stats = "using UnityEngine;

public class Stats : MonoBehaviour
{
    public int health, armor;
    public static int count;
    public const int Max = 3;
    public readonly int id;
    [System.NonSerialized] public int cache;
    [SerializeField] int speed;
    public int Level { get; set; }
}

public class Range : ScriptableObject
{
    public float reach;
}

public class Plain
{
    public int x;
}

public class Window : UnityEditor.EditorWindow
{
    public int tab;
}
";
        let found = written(stats);
        assert_eq!(seen(&found), [(5, Rule::U13), (16, Rule::U13)]);
        assert!(found[0].message.starts_with("`health`, `armor` is public"), "{}", found[0].message);
    }

    #[test]
    fn imgui_is_told_where_a_component_declares_it_and_editor_windows_keep_it() {
        let hud = "using UnityEngine;

public class Hud : MonoBehaviour
{
    void OnGUI()
    {
        GUI.Label(new Rect(0, 0, 100, 20), name);
    }
}

public class Tools : UnityEditor.EditorWindow
{
    void OnGUI()
    {
        GUILayout.Label(titleContent);
    }
}
";
        let found = written(hud);
        assert_eq!(seen(&found), [(5, Rule::U15)], "{found:#?}");
        assert!(found[0].message.starts_with("`Hud.OnGUI` draws with IMGUI"), "{}", found[0].message);
    }

    #[test]
    fn editor_and_test_folders_are_left_alone() {
        for path in ["Assets/Editor/Tool.cs", "Assets/Game/Tests/Probe.cs"] {
            let found = findings(&Index::default(), &Change { path: path.into(), before: None, after: Some(CARELESS.into()) }, &rules(None));
            assert!(found.is_empty(), "{path}: {found:#?}");
        }
        assert!(ships("Assets/Scripts/EditorCamera.cs") && !ships("Assets/editor/Tool.cs"));
    }
}
