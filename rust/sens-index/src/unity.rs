use std::collections::HashMap;

use crate::index::Index;

const BASES: [&str; 7] = ["MonoBehaviour", "ScriptableObject", "Editor", "EditorWindow", "StateMachineBehaviour", "PropertyDrawer", "ScriptableWizard"];

pub const MESSAGES: &[&str] = &[
    "Awake", "OnEnable", "Start", "FixedUpdate", "Update", "LateUpdate", "OnDisable", "OnDestroy", "OnValidate", "Reset", "OnGUI",
    "OnApplicationPause", "OnApplicationFocus", "OnApplicationQuit",
    "OnCollisionEnter", "OnCollisionStay", "OnCollisionExit", "OnCollisionEnter2D", "OnCollisionStay2D", "OnCollisionExit2D",
    "OnTriggerEnter", "OnTriggerStay", "OnTriggerExit", "OnTriggerEnter2D", "OnTriggerStay2D", "OnTriggerExit2D",
    "OnControllerColliderHit", "OnParticleCollision", "OnParticleTrigger", "OnJointBreak", "OnJointBreak2D",
    "OnMouseDown", "OnMouseUp", "OnMouseUpAsButton", "OnMouseEnter", "OnMouseExit", "OnMouseOver", "OnMouseDrag",
    "OnAnimatorMove", "OnAnimatorIK", "OnDrawGizmos", "OnDrawGizmosSelected", "OnBecameVisible", "OnBecameInvisible",
    "OnWillRenderObject", "OnPreCull", "OnPreRender", "OnPostRender", "OnRenderImage", "OnRenderObject",
    "OnTransformParentChanged", "OnTransformChildrenChanged", "OnRectTransformDimensionsChange", "OnCanvasGroupChanged",
    "OnInspectorGUI", "OnSceneGUI", "HasPreviewGUI", "OnPreviewGUI", "CreateInspectorGUI",
    "CreateGUI", "OnFocus", "OnLostFocus", "OnHierarchyChange", "OnProjectChange", "OnSelectionChange", "OnInspectorUpdate",
    "OnStateEnter", "OnStateUpdate", "OnStateExit", "OnStateMove", "OnStateIK", "OnStateMachineEnter", "OnStateMachineExit",
    "GetPropertyHeight", "CreatePropertyGUI",
    "OnWizardCreate", "OnWizardUpdate", "OnWizardOtherButton",
];

const ENTRY_ATTRIBUTES: &[&str] = &[
    "MenuItem", "ContextMenu", "RuntimeInitializeOnLoadMethod", "InitializeOnLoadMethod", "DidReloadScripts", "PostProcessBuild", "PostProcessScene", "OnOpenAsset",
    "Test", "TestCase", "UnityTest", "SetUp", "TearDown", "OneTimeSetUp", "OneTimeTearDown", "UnitySetUp", "UnityTearDown",
];

pub fn is_entry_attribute(name: &str) -> bool {
    ENTRY_ATTRIBUTES.contains(&name.strip_suffix("Attribute").unwrap_or(name))
}

fn is_csharp(file: &str) -> bool {
    file.ends_with(".cs")
}

pub fn derive(found: &mut HashMap<String, &'static str>, types: &[(&str, &[String])]) {
    loop {
        let known = found.len();
        for &(name, bases) in types {
            if found.contains_key(name) {
                continue;
            }
            let root = bases.iter().find_map(|base| BASES.iter().copied().find(|engine| engine == base).or_else(|| found.get(base).copied()));
            if let Some(root) = root {
                found.insert(name.to_string(), root);
            }
        }
        if found.len() == known {
            return;
        }
    }
}

fn roots(index: &Index) -> HashMap<String, &'static str> {
    let types: Vec<(&str, &[String])> = index.symbols.iter().filter(|symbol| symbol.kind == "class" && is_csharp(&symbol.file)).map(|symbol| (symbol.name.as_str(), symbol.bases.as_slice())).collect();
    let mut found = HashMap::new();
    derive(&mut found, &types);
    found
}

fn is_message_of(name: &str, roots: &HashMap<String, &'static str>) -> bool {
    name.split_once('.').is_some_and(|(owner, method)| roots.contains_key(owner) && MESSAGES.contains(&method))
}

pub fn mark(index: &mut Index) {
    let roots = roots(index);
    for symbol in index.symbols.iter_mut().filter(|symbol| is_csharp(&symbol.file)) {
        let component = symbol.kind == "class" && roots.contains_key(&symbol.name);
        let message = symbol.kind == "method" && is_message_of(&symbol.name, &roots);
        symbol.entry |= component || message;
    }
    index.unity = roots;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use crate::query::Engine;

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        fixture::indexed("sens-index-unity", name, files)
    }

    fn is_entry(index: &Index, name: &str) -> bool {
        index.symbols.iter().find(|symbol| symbol.name == name).unwrap_or_else(|| panic!("no symbol {name}")).entry
    }

    const PLAYER: (&str, &str) = (
        "Assets/Scripts/PlayerController.cs",
        "using UnityEngine;\n\npublic class PlayerController : MonoBehaviour\n{\n    void Update()\n    {\n        Move();\n    }\n\n    void Move()\n    {\n    }\n\n    void Unused()\n    {\n    }\n}\n",
    );
    const BOSS: (&str, &str) = ("Assets/Scripts/BossController.cs", "public class BossController : PlayerController\n{\n    void Start()\n    {\n    }\n}\n");

    #[test]
    fn a_component_and_its_messages_are_alive_even_through_a_base_in_another_file() {
        let index = project("components", &[PLAYER, BOSS]);
        assert_eq!(index.unity.get("PlayerController"), Some(&"MonoBehaviour"));
        assert_eq!(index.unity.get("BossController"), Some(&"MonoBehaviour"));
        assert!(is_entry(&index, "PlayerController.Update") && is_entry(&index, "BossController.Start"));
        assert!(!is_entry(&index, "PlayerController.Move") && !is_entry(&index, "PlayerController.Unused"));
        let engine = Engine::new(&index, &index.entry_points);
        let dead: Vec<&str> = engine.dead_code_report(None).candidates.iter().map(|candidate| candidate.symbol.name.as_str()).collect();
        assert_eq!(dead, ["PlayerController.Unused"]);
    }

    #[test]
    fn every_unity_base_counts_however_it_is_written_and_a_plain_class_does_not() {
        let index = project(
            "roots",
            &[(
                "Assets/Scripts/Roots.cs",
                "public class Settings : ScriptableObject { void OnValidate() { } void Tidy() { } }\n\
                 public class SettingsInspector : UnityEditor.Editor { public override void OnInspectorGUI() { } }\n\
                 public abstract class Singleton<T> : MonoBehaviour where T : Component { }\n\
                 public class GameManager : Singleton<GameManager> { void Awake() { } }\n\
                 public class Pool : global::UnityEngine.MonoBehaviour { }\n\
                 public class Base { }\n\
                 public class Plain : Base, System.IDisposable { void Update() { } public void Dispose() { } }\n",
            )],
        );
        let mut roots: Vec<(&str, &str)> = index.unity.iter().map(|(name, root)| (name.as_str(), *root)).collect();
        roots.sort();
        assert_eq!(
            roots,
            [("GameManager", "MonoBehaviour"), ("Pool", "MonoBehaviour"), ("Settings", "ScriptableObject"), ("SettingsInspector", "Editor"), ("Singleton", "MonoBehaviour")]
        );
        assert!(is_entry(&index, "Settings.OnValidate") && is_entry(&index, "SettingsInspector.OnInspectorGUI") && is_entry(&index, "GameManager.Awake"));
        assert!(!is_entry(&index, "Settings.Tidy") && !is_entry(&index, "Plain.Update"));
    }

    #[test]
    fn editor_and_test_attributes_make_a_method_an_entry_point() {
        let index = project(
            "attributes",
            &[(
                "Assets/Checks/Movement.cs",
                "public static class Baking\n{\n    [MenuItem(\"Tools/Bake\")] static void Bake() { }\n    [UnityEngine.ContextMenu(\"Clear\")] static void Clear() { }\n    static void Helper() { }\n}\n\n\
                 public class Movement\n{\n    [Test] public void Moves() { }\n    [UnityTest] public System.Collections.IEnumerator Waits() { yield return null; }\n    [SetUpAttribute] public void Prepare() { }\n}\n",
            )],
        );
        for name in ["Baking.Bake", "Baking.Clear", "Movement.Moves", "Movement.Waits", "Movement.Prepare"] {
            assert!(is_entry(&index, name), "{name}");
        }
        assert!(!is_entry(&index, "Baking.Helper"));
    }

    #[test]
    fn the_attribute_suffix_is_optional() {
        assert!(is_entry_attribute("MenuItem") && is_entry_attribute("MenuItemAttribute") && is_entry_attribute("UnityTest"));
        assert!(!is_entry_attribute("Serializable") && !is_entry_attribute("Attribute"));
    }
}
