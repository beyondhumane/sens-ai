## Unity

This is a Unity project, and Sens reads its C# as Unity code: which classes are components, which methods Unity calls and which code runs every frame. `file_outline` marks the methods that run every frame and how many calls away from one they are.

Before you write a script, go down this ladder too:

1. Is it data? Values someone tunes, such as stats, waves, items or levels, belong in a `ScriptableObject` asset, not in code or on a component.
2. Does the project already have the component or the system? Ask `already_exists` before adding a manager, a pool or a singleton.
3. Will it run every frame? Whatever `Update`, `FixedUpdate`, `LateUpdate`, `OnGUI` or a coroutine that yields every frame reaches is the hot path, and it costs every frame.

In the hot path:

- Look components up once in `Awake` and keep them in fields: no `GetComponent`, `Camera.main` or `Find` there.
- Do not allocate: no LINQ, new lists or arrays, string building or `new WaitForSeconds`. Keep buffers in fields and use the `NonAlloc` physics queries.
- Name Animator and shader properties by id: `Animator.StringToHash` or `Shader.PropertyToID`, kept in a `static readonly int`.
- Take objects from a pool instead of calling `Instantiate` and `Destroy`.
- Keep logs out of the build with `#if UNITY_EDITOR || DEVELOPMENT_BUILD`.

Everywhere:

- `[SerializeField] private` instead of `public` for what only the Inspector sets.
- `CompareTag` instead of `tag ==`, and a direct reference or an event instead of `SendMessage`.
- Delete empty Unity messages, such as an empty `Update` or `Start`: Unity calls them anyway.
- `Task`, or `Awaitable` from Unity 2023.1, instead of `async void` outside event handlers.
- UI Toolkit or uGUI for interfaces, not `OnGUI`.
- Never edit `Library/`, `Temp/` or `.meta` files by hand, and move or rename an asset together with its `.meta`.
