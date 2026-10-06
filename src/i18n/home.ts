import { copy } from "./locales";

export type Piece = string | { strong: string } | { code: string } | { link: string };

interface Chapter {
  title: string;
  text: string;
  replay: string;
}

interface Figure {
  figure: string;
  said: string;
}

interface Check {
  title: string;
  text: string;
}

export const home = copy({
  en: {
    title: "Sens · Claude Code that reuses what you have, and writes less",
    description:
      "A desktop app for Claude Code on Windows. Every step the agent takes on screen, and every change checked before it reaches the disk: 12% less code and 18% fewer tokens in our benchmark. Open source, MIT.",
    structured:
      "A desktop app for Claude Code that shows every step the agent takes and checks every change before it reaches the disk, so Claude reuses what a project has and writes less.",
    lines: [["Less", "noise."], ["More", "sense."]] as string[][],
    lead: "Claude Code that reuses what you have, and writes less.",
    proof: [{ strong: "12%" }, " less code and ", { strong: "18%" }, " fewer tokens in our benchmark. ", { link: "How" }] as Piece[],
    viewOnGithub: "View on GitHub",
    phoneNote: "Sens runs on Windows 10 and 11. Open this page on your PC to install it.",
    windowDescription:
      "Beside the headline, the Sens window shows a finished Claude Code session: the request at the top, the agent's work folded into one line, the answer, and the changed files with their diff.",
    demoLabel: "How it works",
    demoTitle: "A chat that shows its work.",
    demoLead:
      "A desktop app for Claude Code. Every step the agent takes on screen, with the project's files, changes, terminal and browser beside it.",
    replayTag: (minor: string) => `Replay · recreated from Sens ${minor}`,
    chaptersLabel: "Chapters",
    chapterLabel: (number: string, title: string) => `Chapter ${number}: ${title}`,
    replay: "Replay",
    replayChapter: (number: string) => `Replay chapter ${number}`,
    chapters: [
      {
        title: "You ask.",
        text: "Type it, or say it. The request stays at the top of the thread, with the project and branch it works in.",
        replay: "In the replay, the request sits at the top of the thread, with the orbit folder and the main branch above the message box.",
      },
      {
        title: "Sens shows the work.",
        text: "Every tool call is a step, in order. The first is Sens's own: the code your project already has for what you asked.",
        replay:
          "In the replay, the line Worked opens into its steps. The first says Sens already has greetingFor, used 3 times; then Claude reads, searches, runs the tests and edits. The first test run comes into focus.",
      },
      {
        title: "Open any step.",
        text: "A command opens to its output, in its own program's colours. A file and line open the file.",
        replay: "In the replay, the step Run npm test opens to its output: 2 test files and 10 tests passed.",
      },
      {
        title: "Sens checks it.",
        text: "Every change is checked before it reaches the disk. Claude wrote a greeting helper the project already had; Sens stopped the write, showed the original, and Claude used it.",
        replay:
          "In the replay, the step Sens stopped a write opens: it copies existing code at src/header.tsx line 7, and the function greetingFor is already in src/greeting.ts. Next, Claude thinks: using greetingFor instead.",
      },
      {
        title: "See what changed.",
        text: "Each edit lands in Changes as a diff, file by file, against the last commit.",
        replay:
          "In the replay, the path src/header.tsx travels from its Edit step to Changes, where header.tsx shows 4 lines added and 2 removed, calling greetingFor.",
      },
      {
        title: "Read the result.",
        text: "When the reply is done, the work folds into one line: what Claude did, and what Sens stopped, reused and approved. A failure is never folded away.",
        replay:
          "In the replay, the steps fold back into one line that ends with Sens: 1 stop, 1 reused, approved. The answer comes forward: the header takes an optional greeting and reuses greetingFor, and the suite passes with 12 tests.",
      },
    ] as Chapter[],
    engineLabel: "What Sens checks",
    engineTitle: "Less code for the same work.",
    engineLead: "Sens stands between Claude Code and your project. It runs in every session, and Claude can't switch it off or go around it.",
    measured: [
      { figure: "12%", said: "less code for the same features" },
      { figure: "18%", said: "fewer tokens" },
      { figure: "90 of 90", said: "tasks accepted, as without Sens" },
    ] as Figure[],
    engineNote: "After 30 chained tasks on one project with Claude Sonnet 5.5, in our benchmark.",
    methodLink: "Method and results",
    checks: [
      {
        title: "It shows what you already have.",
        text: "With every message, Sens shows Claude the code in your project that relates to what you asked, so it reuses it instead of writing it again.",
      },
      {
        title: "It checks every change before it lands.",
        text: "Copies of code you already have, renamed ones too, code left unused and new comments are stopped before they reach the disk, with what to do instead. A new dependency or removed tests wait for you.",
      },
      {
        title: "It reads the turn again.",
        text: "When the changes pass, a reviewer looks for abstractions used once, fixes to the symptom instead of the cause, and rewrites of what the project already gives. Its notes reach you and Claude; they stop nothing.",
      },
      {
        title: "You have the last word.",
        text: "If Claude can't get its changes approved in three tries, the turn is held. Accept them, undo them without touching your git, or ask Claude to fix them.",
      },
    ] as Check[],
    languages:
      "Thirty languages. TypeScript and JavaScript with Vue and Svelte, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, C, C++, Swift, Dart, Scala, F#, Lua, Elixir, Gleam and Zig in full; eleven more checked for copies and comments.",
    closingTitle: "Give Claude Code a window.",
    exploreOnGithub: "Explore on GitHub",
    windowsVersions: "Windows 10 or 11",
    claudeAccount: "A Claude account",
    unsigned: [
      "The installer isn't code-signed yet, so Windows SmartScreen will warn you. Compare it with the SHA-256 in the ",
      { link: "release notes" },
      ".",
    ] as Piece[],
  },
  es: {
    title: "Sens · Claude Code que reutiliza lo que ya tienes y escribe menos",
    description:
      "Una app de escritorio para Claude Code en Windows. Cada paso del agente en pantalla y cada cambio revisado antes de llegar al disco: un 12 % menos de código y un 18 % menos de tokens en nuestro banco de pruebas. Código abierto, MIT.",
    structured:
      "Una app de escritorio para Claude Code que muestra cada paso del agente y revisa cada cambio antes de que llegue al disco, para que Claude reutilice lo que ya tiene el proyecto y escriba menos.",
    lines: [["Menos", "ruido."], ["Más", "sentido."]] as string[][],
    lead: "Claude Code que reutiliza lo que ya tienes y escribe menos.",
    proof: [{ strong: "12 %" }, " menos de código y ", { strong: "18 %" }, " menos de tokens en nuestro banco de pruebas. ", { link: "Cómo" }] as Piece[],
    viewOnGithub: "Ver en GitHub",
    phoneNote: "Sens funciona en Windows 10 y 11. Abre esta página en tu ordenador para instalarlo.",
    windowDescription:
      "Junto al titular, la ventana de Sens muestra una sesión de Claude Code terminada: la petición arriba, el trabajo del agente plegado en una línea, la respuesta y los ficheros cambiados con su diff.",
    demoLabel: "Cómo funciona",
    demoTitle: "Un chat que enseña su trabajo.",
    demoLead:
      "Una app de escritorio para Claude Code. Cada paso del agente en pantalla, con los ficheros, los cambios, la terminal y el navegador del proyecto al lado.",
    replayTag: (minor: string) => `Repetición · recreada a partir de Sens ${minor}`,
    chaptersLabel: "Capítulos",
    chapterLabel: (number: string, title: string) => `Capítulo ${number}: ${title}`,
    replay: "Repetir",
    replayChapter: (number: string) => `Repetir el capítulo ${number}`,
    chapters: [
      {
        title: "Tú pides.",
        text: "Escríbelo o dilo. La petición se queda arriba del hilo, con el proyecto y la rama en los que trabaja.",
        replay: "En la repetición, la petición está arriba del hilo, con la carpeta orbit y la rama main sobre el cuadro del mensaje.",
      },
      {
        title: "Sens enseña el trabajo.",
        text: "Cada llamada a una herramienta es un paso, en orden. El primero es de Sens: el código que tu proyecto ya tiene para lo que pediste.",
        replay:
          "En la repetición, la línea Worked se abre en sus pasos. El primero dice que Sens ya tiene greetingFor, usado 3 veces; luego Claude lee, busca, ejecuta los tests y edita. El foco cae en la primera ejecución de los tests.",
      },
      {
        title: "Abre cualquier paso.",
        text: "Un comando se abre en su salida, con los colores de su propio programa. Un fichero y una línea abren el fichero.",
        replay: "En la repetición, el paso Run npm test se abre en su salida: 2 ficheros de tests y 10 tests superados.",
      },
      {
        title: "Sens lo revisa.",
        text: "Cada cambio se revisa antes de llegar al disco. Claude escribió un helper de saludo que el proyecto ya tenía; Sens paró la escritura, enseñó el original y Claude lo usó.",
        replay:
          "En la repetición se abre el paso Sens stopped a write: copia código que ya existe en src/header.tsx, línea 7, y la función greetingFor ya está en src/greeting.ts. Después, Claude piensa: usar greetingFor.",
      },
      {
        title: "Mira qué cambió.",
        text: "Cada edición llega a Changes como un diff, fichero a fichero, frente al último commit.",
        replay:
          "En la repetición, la ruta src/header.tsx viaja de su paso Edit a Changes, donde header.tsx muestra 4 líneas añadidas y 2 quitadas, con la llamada a greetingFor.",
      },
      {
        title: "Lee el resultado.",
        text: "Cuando la respuesta termina, el trabajo se pliega en una línea: lo que hizo Claude y lo que Sens paró, reutilizó y aprobó. Un fallo nunca se pliega.",
        replay:
          "En la repetición, los pasos se pliegan en una línea que acaba en Sens: 1 stop, 1 reused, approved. La respuesta pasa al frente: el encabezado acepta un saludo opcional y reutiliza greetingFor, y la batería pasa con 12 tests.",
      },
    ] as Chapter[],
    engineLabel: "Qué revisa Sens",
    engineTitle: "Menos código para el mismo trabajo.",
    engineLead: "Sens se pone entre Claude Code y tu proyecto. Funciona en cada sesión, y Claude no puede apagarlo ni rodearlo.",
    measured: [
      { figure: "12 %", said: "menos código para las mismas funciones" },
      { figure: "18 %", said: "menos tokens" },
      { figure: "90 de 90", said: "tareas aceptadas, igual que sin Sens" },
    ] as Figure[],
    engineNote: "Tras 30 tareas encadenadas sobre un proyecto con Claude Sonnet 5.5, en nuestro banco de pruebas.",
    methodLink: "Método y resultados",
    checks: [
      {
        title: "Te enseña lo que ya tienes.",
        text: "Con cada mensaje, Sens le enseña a Claude el código de tu proyecto relacionado con lo que pediste, para que lo reutilice en vez de escribirlo otra vez.",
      },
      {
        title: "Revisa cada cambio antes de que llegue.",
        text: "Las copias de código que ya tienes, también las renombradas, el código sin usar y los comentarios nuevos se paran antes de llegar al disco, con lo que hay que hacer en su lugar. Una dependencia nueva o unos tests quitados te esperan a ti.",
      },
      {
        title: "Vuelve a leer el turno.",
        text: "Cuando los cambios pasan, un revisor busca abstracciones de un solo uso, arreglos del síntoma en vez de la causa y reescrituras de lo que el proyecto ya da. Sus notas os llegan a ti y a Claude; no paran nada.",
      },
      {
        title: "La última palabra es tuya.",
        text: "Si Claude no consigue que le aprueben los cambios en tres intentos, el turno queda retenido. Acéptalos, deshazlos sin tocar tu git o pídele a Claude que los arregle.",
      },
    ] as Check[],
    languages:
      "Treinta lenguajes. TypeScript y JavaScript con Vue y Svelte, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, C, C++, Swift, Dart, Scala, F#, Lua, Elixir, Gleam y Zig completos; once más revisados en copias y comentarios.",
    closingTitle: "Dale una ventana a Claude Code.",
    exploreOnGithub: "Explorar en GitHub",
    windowsVersions: "Windows 10 u 11",
    claudeAccount: "Una cuenta de Claude",
    unsigned: [
      "El instalador aún no está firmado, así que Windows SmartScreen te avisará. Compáralo con el SHA-256 de las ",
      { link: "notas de la versión" },
      ".",
    ] as Piece[],
  },
  fr: {
    title: "Sens · Claude Code qui réutilise ce que vous avez déjà et écrit moins",
    description:
      "Une application de bureau pour Claude Code sous Windows. Chaque étape de l'agent à l'écran, et chaque modification vérifiée avant d'atteindre le disque : 12 % de code en moins et 18 % de tokens en moins dans notre banc d'essai. Open source, MIT.",
    structured:
      "Une application de bureau pour Claude Code qui montre chaque étape de l'agent et vérifie chaque modification avant qu'elle n'atteigne le disque, pour que Claude réutilise ce que le projet contient et écrive moins.",
    lines: [["Moins", "de bruit."], ["Plus", "de sens."]] as string[][],
    lead: "Claude Code qui réutilise ce que vous avez déjà et écrit moins.",
    proof: [{ strong: "12 %" }, " de code en moins et ", { strong: "18 %" }, " de tokens en moins dans notre banc d'essai. ", { link: "Comment" }] as Piece[],
    viewOnGithub: "Voir sur GitHub",
    phoneNote: "Sens fonctionne sous Windows 10 et 11. Ouvrez cette page sur votre PC pour l'installer.",
    windowDescription:
      "À côté du titre, la fenêtre de Sens montre une session Claude Code terminée : la demande en haut, le travail de l'agent replié en une ligne, la réponse et les fichiers modifiés avec leur diff.",
    demoLabel: "Comment ça marche",
    demoTitle: "Un chat qui montre son travail.",
    demoLead:
      "Une application de bureau pour Claude Code. Chaque étape de l'agent à l'écran, avec les fichiers, les modifications, le terminal et le navigateur du projet à côté.",
    replayTag: (minor: string) => `Reconstitution · d'après Sens ${minor}`,
    chaptersLabel: "Chapitres",
    chapterLabel: (number: string, title: string) => `Chapitre ${number} : ${title}`,
    replay: "Rejouer",
    replayChapter: (number: string) => `Rejouer le chapitre ${number}`,
    chapters: [
      {
        title: "Vous demandez.",
        text: "Écrivez-le ou dites-le. La demande reste en haut du fil, avec le projet et la branche où elle s'applique.",
        replay: "Dans la reconstitution, la demande est en haut du fil, avec le dossier orbit et la branche main au-dessus du champ de message.",
      },
      {
        title: "Sens montre le travail.",
        text: "Chaque appel d'outil est une étape, dans l'ordre. La première vient de Sens : le code que votre projet a déjà pour ce que vous demandez.",
        replay:
          "Dans la reconstitution, la ligne Worked s'ouvre sur ses étapes. La première indique que Sens a déjà greetingFor, utilisé 3 fois ; puis Claude lit, cherche, lance les tests et modifie. Le premier lancement des tests passe au premier plan.",
      },
      {
        title: "Ouvrez n'importe quelle étape.",
        text: "Une commande s'ouvre sur sa sortie, avec les couleurs de son propre programme. Un fichier et une ligne ouvrent le fichier.",
        replay: "Dans la reconstitution, l'étape Run npm test s'ouvre sur sa sortie : 2 fichiers de tests et 10 tests réussis.",
      },
      {
        title: "Sens vérifie.",
        text: "Chaque modification est vérifiée avant d'atteindre le disque. Claude a écrit une fonction de salutation que le projet avait déjà ; Sens a arrêté l'écriture, montré l'original, et Claude l'a utilisé.",
        replay:
          "Dans la reconstitution, l'étape Sens stopped a write s'ouvre : elle copie du code existant dans src/header.tsx, ligne 7, et la fonction greetingFor existe déjà dans src/greeting.ts. Ensuite, Claude réfléchit : utiliser greetingFor.",
      },
      {
        title: "Voyez ce qui a changé.",
        text: "Chaque modification arrive dans Changes sous forme de diff, fichier par fichier, par rapport au dernier commit.",
        replay:
          "Dans la reconstitution, le chemin src/header.tsx passe de son étape Edit à Changes, où header.tsx montre 4 lignes ajoutées et 2 supprimées, avec l'appel à greetingFor.",
      },
      {
        title: "Lisez le résultat.",
        text: "Quand la réponse est terminée, le travail se replie en une ligne : ce que Claude a fait, et ce que Sens a arrêté, réutilisé et approuvé. Un échec n'est jamais replié.",
        replay:
          "Dans la reconstitution, les étapes se replient en une ligne qui se termine par Sens: 1 stop, 1 reused, approved. La réponse passe au premier plan : l'en-tête accepte une salutation facultative et réutilise greetingFor, et la suite passe avec 12 tests.",
      },
    ] as Chapter[],
    engineLabel: "Ce que Sens vérifie",
    engineTitle: "Moins de code pour le même travail.",
    engineLead:
      "Sens se place entre Claude Code et votre projet. Il fonctionne dans chaque session, et Claude ne peut ni le désactiver ni le contourner.",
    measured: [
      { figure: "12 %", said: "de code en moins pour les mêmes fonctionnalités" },
      { figure: "18 %", said: "de tokens en moins" },
      { figure: "90 sur 90", said: "tâches acceptées, comme sans Sens" },
    ] as Figure[],
    engineNote: "Après 30 tâches enchaînées sur un projet avec Claude Sonnet 5.5, dans notre banc d'essai.",
    methodLink: "Méthode et résultats",
    checks: [
      {
        title: "Il montre ce que vous avez déjà.",
        text: "À chaque message, Sens montre à Claude le code de votre projet lié à votre demande, pour qu'il le réutilise au lieu de le réécrire.",
      },
      {
        title: "Il vérifie chaque modification avant qu'elle n'arrive.",
        text: "Les copies de code existant, même renommées, le code laissé inutilisé et les nouveaux commentaires sont arrêtés avant d'atteindre le disque, avec ce qu'il faut faire à la place. Une nouvelle dépendance ou des tests supprimés vous attendent.",
      },
      {
        title: "Il relit le tour.",
        text: "Quand les modifications passent, un relecteur cherche les abstractions à usage unique, les corrections du symptôme plutôt que de la cause et les réécritures de ce que le projet fournit déjà. Ses remarques vous parviennent, à vous et à Claude ; elles n'arrêtent rien.",
      },
      {
        title: "Vous avez le dernier mot.",
        text: "Si Claude n'obtient pas l'approbation de ses modifications en trois essais, le tour est retenu. Acceptez-les, annulez-les sans toucher à votre git, ou demandez à Claude de les corriger.",
      },
    ] as Check[],
    languages:
      "Trente langages. TypeScript et JavaScript avec Vue et Svelte, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, C, C++, Swift, Dart, Scala, F#, Lua, Elixir, Gleam et Zig complets ; onze autres vérifiés pour les copies et les commentaires.",
    closingTitle: "Donnez une fenêtre à Claude Code.",
    exploreOnGithub: "Explorer sur GitHub",
    windowsVersions: "Windows 10 ou 11",
    claudeAccount: "Un compte Claude",
    unsigned: [
      "Le programme d'installation n'est pas encore signé, Windows SmartScreen vous avertira donc. Comparez-le avec le SHA-256 des ",
      { link: "notes de version" },
      ".",
    ] as Piece[],
  },
  de: {
    title: "Sens · Claude Code, das wiederverwendet, was du hast, und weniger schreibt",
    description:
      "Eine Desktop-App für Claude Code unter Windows. Jeder Schritt des Agenten auf dem Bildschirm und jede Änderung geprüft, bevor sie auf der Festplatte landet: 12 % weniger Code und 18 % weniger Tokens in unserem Benchmark. Open Source, MIT.",
    structured:
      "Eine Desktop-App für Claude Code, die jeden Schritt des Agenten zeigt und jede Änderung prüft, bevor sie auf der Festplatte landet, damit Claude wiederverwendet, was das Projekt hat, und weniger schreibt.",
    lines: [["Weniger", "Lärm."], ["Mehr", "Sinn."]] as string[][],
    lead: "Claude Code, das wiederverwendet, was du hast, und weniger schreibt.",
    proof: [{ strong: "12 %" }, " weniger Code und ", { strong: "18 %" }, " weniger Tokens in unserem Benchmark. ", { link: "Wie" }] as Piece[],
    viewOnGithub: "Auf GitHub ansehen",
    phoneNote: "Sens läuft unter Windows 10 und 11. Öffne diese Seite auf deinem PC, um es zu installieren.",
    windowDescription:
      "Neben der Überschrift zeigt das Sens-Fenster eine abgeschlossene Claude-Code-Sitzung: oben die Anfrage, die Arbeit des Agenten in einer Zeile zusammengefasst, die Antwort und die geänderten Dateien mit ihrem Diff.",
    demoLabel: "So funktioniert es",
    demoTitle: "Ein Chat, der seine Arbeit zeigt.",
    demoLead:
      "Eine Desktop-App für Claude Code. Jeder Schritt des Agenten auf dem Bildschirm, mit Dateien, Änderungen, Terminal und Browser des Projekts daneben.",
    replayTag: (minor: string) => `Wiedergabe · nachgebaut aus Sens ${minor}`,
    chaptersLabel: "Kapitel",
    chapterLabel: (number: string, title: string) => `Kapitel ${number}: ${title}`,
    replay: "Wiederholen",
    replayChapter: (number: string) => `Kapitel ${number} wiederholen`,
    chapters: [
      {
        title: "Du fragst.",
        text: "Schreib es oder sag es. Die Anfrage bleibt oben im Verlauf, mit dem Projekt und dem Branch, in dem sie arbeitet.",
        replay: "In der Wiedergabe steht die Anfrage oben im Verlauf, mit dem Ordner orbit und dem Branch main über dem Nachrichtenfeld.",
      },
      {
        title: "Sens zeigt die Arbeit.",
        text: "Jeder Werkzeugaufruf ist ein Schritt, der Reihe nach. Der erste kommt von Sens: der Code, den dein Projekt für deine Anfrage schon hat.",
        replay:
          "In der Wiedergabe öffnet sich die Zeile Worked in ihre Schritte. Der erste sagt, dass Sens greetingFor schon hat, dreimal benutzt; dann liest, sucht, testet und bearbeitet Claude. Der erste Testlauf rückt in den Fokus.",
      },
      {
        title: "Öffne jeden Schritt.",
        text: "Ein Befehl öffnet sich zu seiner Ausgabe, in den Farben seines eigenen Programms. Datei und Zeile öffnen die Datei.",
        replay: "In der Wiedergabe öffnet sich der Schritt Run npm test zu seiner Ausgabe: 2 Testdateien und 10 Tests bestanden.",
      },
      {
        title: "Sens prüft es.",
        text: "Jede Änderung wird geprüft, bevor sie auf der Festplatte landet. Claude schrieb eine Begrüßungsfunktion, die das Projekt schon hatte; Sens stoppte das Schreiben, zeigte das Original, und Claude nutzte es.",
        replay:
          "In der Wiedergabe öffnet sich der Schritt Sens stopped a write: Er kopiert vorhandenen Code in src/header.tsx, Zeile 7, und die Funktion greetingFor gibt es schon in src/greeting.ts. Danach denkt Claude: stattdessen greetingFor nutzen.",
      },
      {
        title: "Sieh, was sich geändert hat.",
        text: "Jede Bearbeitung landet in Changes als Diff, Datei für Datei, gegenüber dem letzten Commit.",
        replay:
          "In der Wiedergabe wandert der Pfad src/header.tsx von seinem Edit-Schritt zu Changes, wo header.tsx 4 hinzugefügte und 2 entfernte Zeilen zeigt, mit dem Aufruf von greetingFor.",
      },
      {
        title: "Lies das Ergebnis.",
        text: "Wenn die Antwort fertig ist, fasst sich die Arbeit in einer Zeile zusammen: was Claude getan hat und was Sens gestoppt, wiederverwendet und freigegeben hat. Ein Fehler wird nie eingeklappt.",
        replay:
          "In der Wiedergabe klappen die Schritte in eine Zeile zusammen, die mit Sens: 1 stop, 1 reused, approved endet. Die Antwort rückt nach vorn: Der Header nimmt eine optionale Begrüßung an und nutzt greetingFor, und die Tests laufen mit 12 Tests durch.",
      },
    ] as Chapter[],
    engineLabel: "Was Sens prüft",
    engineTitle: "Weniger Code für dieselbe Arbeit.",
    engineLead:
      "Sens steht zwischen Claude Code und deinem Projekt. Es läuft in jeder Sitzung, und Claude kann es weder abschalten noch umgehen.",
    measured: [
      { figure: "12 %", said: "weniger Code für dieselben Funktionen" },
      { figure: "18 %", said: "weniger Tokens" },
      { figure: "90 von 90", said: "Aufgaben angenommen, wie ohne Sens" },
    ] as Figure[],
    engineNote: "Nach 30 verketteten Aufgaben an einem Projekt mit Claude Sonnet 5.5, in unserem Benchmark.",
    methodLink: "Methode und Ergebnisse",
    checks: [
      {
        title: "Es zeigt, was du schon hast.",
        text: "Mit jeder Nachricht zeigt Sens Claude den Code deines Projekts, der zu deiner Anfrage passt, damit es ihn wiederverwendet, statt ihn neu zu schreiben.",
      },
      {
        title: "Es prüft jede Änderung, bevor sie landet.",
        text: "Kopien von vorhandenem Code, auch umbenannte, ungenutzter Code und neue Kommentare werden gestoppt, bevor sie auf der Festplatte landen, mit dem, was stattdessen zu tun ist. Eine neue Abhängigkeit oder entfernte Tests warten auf dich.",
      },
      {
        title: "Es liest den Durchgang noch einmal.",
        text: "Wenn die Änderungen bestehen, sucht ein Prüfer nach Abstraktionen mit nur einer Verwendung, Korrekturen am Symptom statt an der Ursache und Nachbauten dessen, was das Projekt schon bietet. Seine Hinweise gehen an dich und an Claude; sie stoppen nichts.",
      },
      {
        title: "Das letzte Wort hast du.",
        text: "Bekommt Claude seine Änderungen nach drei Versuchen nicht freigegeben, wird der Durchgang angehalten. Nimm sie an, mach sie rückgängig, ohne dein Git anzufassen, oder bitte Claude, sie zu korrigieren.",
      },
    ] as Check[],
    languages:
      "Dreißig Sprachen. TypeScript und JavaScript mit Vue und Svelte, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, C, C++, Swift, Dart, Scala, F#, Lua, Elixir, Gleam und Zig vollständig; elf weitere auf Kopien und Kommentare geprüft.",
    closingTitle: "Gib Claude Code ein Fenster.",
    exploreOnGithub: "Auf GitHub entdecken",
    windowsVersions: "Windows 10 oder 11",
    claudeAccount: "Ein Claude-Konto",
    unsigned: [
      "Der Installer ist noch nicht signiert, deshalb warnt Windows SmartScreen. Vergleiche ihn mit dem SHA-256 in den ",
      { link: "Versionshinweisen" },
      ".",
    ] as Piece[],
  },
  ja: {
    title: "Sens · 今あるものを再利用し、書くコードを減らす Claude Code",
    description:
      "Windows 向けの Claude Code デスクトップアプリ。エージェントの各ステップを画面に表示し、すべての変更をディスクに書き込まれる前に確認します。ベンチマークではコードが 12% 減り、トークンが 18% 減りました。オープンソース、MIT。",
    structured:
      "エージェントの各ステップを表示し、すべての変更をディスクに書き込まれる前に確認する Claude Code のデスクトップアプリ。Claude はプロジェクトにあるものを再利用し、書くコードが減ります。",
    lines: [["ノイズは少なく。"], ["意味は多く。"]] as string[][],
    lead: "今あるものを再利用し、書くコードを減らす Claude Code。",
    proof: ["ベンチマークでコードは ", { strong: "12%" }, " 減、トークンは ", { strong: "18%" }, " 減。", { link: "その方法" }] as Piece[],
    viewOnGithub: "GitHub で見る",
    phoneNote: "Sens は Windows 10 と 11 で動作します。インストールするには PC でこのページを開いてください。",
    windowDescription:
      "見出しの横の Sens ウィンドウには、完了した Claude Code セッションが表示されています。上にリクエスト、1 行にまとめられたエージェントの作業、返信、そして差分付きの変更ファイルです。",
    demoLabel: "使い方",
    demoTitle: "作業を見せるチャット。",
    demoLead: "Claude Code のためのデスクトップアプリ。エージェントの各ステップを画面に表示し、プロジェクトのファイル、変更、ターミナル、ブラウザーを横に並べます。",
    replayTag: (minor: string) => `リプレイ · Sens ${minor} から再現`,
    chaptersLabel: "チャプター",
    chapterLabel: (number: string, title: string) => `チャプター ${number}：${title}`,
    replay: "もう一度",
    replayChapter: (number: string) => `チャプター ${number} をもう一度再生`,
    chapters: [
      {
        title: "頼む。",
        text: "入力しても、話してもかまいません。リクエストはスレッドの一番上に残り、作業するプロジェクトとブランチも表示されます。",
        replay: "リプレイでは、リクエストがスレッドの一番上にあり、メッセージ欄の上に orbit フォルダーと main ブランチが表示されています。",
      },
      {
        title: "Sens が作業を見せる。",
        text: "ツールの呼び出しはすべて、順番どおりのステップになります。最初のステップは Sens 自身のもので、頼んだことに対してプロジェクトにすでにあるコードを示します。",
        replay:
          "リプレイでは、Worked の行が開いてステップが並びます。最初は Sens がすでに greetingFor（3 回使用）を持っていると示し、その後 Claude が読み、検索し、テストを実行し、編集します。最初のテスト実行に焦点が移ります。",
      },
      {
        title: "どのステップも開ける。",
        text: "コマンドを開くと、そのプログラム自身の色で出力が表示されます。ファイルと行を開くとファイルが開きます。",
        replay: "リプレイでは、Run npm test のステップが開いて出力が表示されます。テストファイル 2 件、テスト 10 件が成功です。",
      },
      {
        title: "Sens が確認する。",
        text: "すべての変更はディスクに書き込まれる前に確認されます。Claude はプロジェクトにすでにあるあいさつの関数を書きました。Sens は書き込みを止めて元の関数を示し、Claude はそれを使いました。",
        replay:
          "リプレイでは、Sens stopped a write のステップが開きます。src/header.tsx の 7 行目で既存のコードをコピーしており、関数 greetingFor はすでに src/greeting.ts にあります。続いて Claude は「代わりに greetingFor を使う」と考えます。",
      },
      {
        title: "何が変わったかを見る。",
        text: "編集はすべて、直前のコミットとの差分として、ファイルごとに Changes に届きます。",
        replay:
          "リプレイでは、パス src/header.tsx が Edit のステップから Changes へ移り、header.tsx に 4 行の追加と 2 行の削除が表示されます。greetingFor を呼び出しています。",
      },
      {
        title: "結果を読む。",
        text: "返信が終わると、作業は 1 行にまとまります。Claude がしたこと、そして Sens が止めたもの、再利用したもの、承認したものです。失敗は決して折りたたまれません。",
        replay:
          "リプレイでは、ステップが 1 行にまとまり、末尾は Sens: 1 stop, 1 reused, approved です。返信が前面に出ます。ヘッダーは省略可能なあいさつを受け取り greetingFor を再利用し、テストは 12 件すべて成功します。",
      },
    ] as Chapter[],
    engineLabel: "Sens が確認すること",
    engineTitle: "同じ仕事を、より少ないコードで。",
    engineLead: "Sens は Claude Code とあなたのプロジェクトのあいだに立ちます。すべてのセッションで動き、Claude はそれを止めることも回避することもできません。",
    measured: [
      { figure: "12%", said: "同じ機能でのコードの削減" },
      { figure: "18%", said: "トークンの削減" },
      { figure: "90 / 90", said: "件のタスクを承認（Sens なしと同じ）" },
    ] as Figure[],
    engineNote: "ベンチマークにて、Claude Sonnet 5.5 で 1 つのプロジェクトに 30 件のタスクを連続して行った結果。",
    methodLink: "方法と結果",
    checks: [
      {
        title: "すでにあるものを見せる。",
        text: "メッセージのたびに、Sens は依頼に関係するプロジェクトのコードを Claude に示します。Claude は書き直す代わりにそれを再利用します。",
      },
      {
        title: "書き込まれる前に、すべての変更を確認する。",
        text: "既存コードのコピー（名前を変えたものも含む）、使われないまま残ったコード、新しいコメントは、ディスクに書き込まれる前に止められ、代わりにすべきことが示されます。新しい依存関係や削除されたテストは、あなたの判断を待ちます。",
      },
      {
        title: "ターンをもう一度読む。",
        text: "変更が通ると、レビュアーが 1 回しか使われない抽象化、原因ではなく症状への修正、プロジェクトがすでに提供しているものの作り直しを探します。その指摘はあなたと Claude に届きますが、何も止めません。",
      },
      {
        title: "最後に決めるのはあなた。",
        text: "Claude が 3 回の試行で変更の承認を得られなければ、ターンは保留になります。受け入れる、git に触れずに元に戻す、Claude に修正を頼む、のいずれかを選べます。",
      },
    ] as Check[],
    languages:
      "30 の言語に対応。TypeScript と JavaScript（Vue、Svelte を含む）、Python、Go、Rust、Java、Kotlin、C#、PHP、Ruby、C、C++、Swift、Dart、Scala、F#、Lua、Elixir、Gleam、Zig は完全対応。ほかの 11 言語はコピーとコメントを確認します。",
    closingTitle: "Claude Code にウィンドウを。",
    exploreOnGithub: "GitHub で見る",
    windowsVersions: "Windows 10 または 11",
    claudeAccount: "Claude アカウント",
    unsigned: [
      "インストーラーはまだコード署名されていないため、Windows SmartScreen が警告を表示します。",
      { link: "リリースノート" },
      "の SHA-256 と照合してください。",
    ] as Piece[],
  },
  zh: {
    title: "Sens · 复用你已有的代码、写得更少的 Claude Code",
    description:
      "Windows 上的 Claude Code 桌面应用。代理的每一步都显示在屏幕上，每个更改在写入磁盘前都会被检查：在我们的基准测试中，代码减少 12%，token 减少 18%。开源，MIT。",
    structured: "一款 Claude Code 桌面应用，显示代理的每一步，并在每个更改写入磁盘前进行检查，让 Claude 复用项目中已有的代码、写得更少。",
    lines: [["少些噪音。"], ["多些意义。"]] as string[][],
    lead: "复用你已有的代码、写得更少的 Claude Code。",
    proof: ["在我们的基准测试中，代码减少 ", { strong: "12%" }, "，token 减少 ", { strong: "18%" }, "。", { link: "怎么做到的" }] as Piece[],
    viewOnGithub: "在 GitHub 上查看",
    phoneNote: "Sens 运行于 Windows 10 和 11。请在电脑上打开此页面进行安装。",
    windowDescription: "标题旁的 Sens 窗口展示了一个已完成的 Claude Code 会话：顶部是请求，代理的工作折叠成一行，然后是回复，以及带差异的已更改文件。",
    demoLabel: "工作方式",
    demoTitle: "一个展示自己工作的聊天。",
    demoLead: "一款 Claude Code 桌面应用。代理的每一步都显示在屏幕上，项目的文件、更改、终端和浏览器就在旁边。",
    replayTag: (minor: string) => `回放 · 根据 Sens ${minor} 重现`,
    chaptersLabel: "章节",
    chapterLabel: (number: string, title: string) => `第 ${number} 章：${title}`,
    replay: "重播",
    replayChapter: (number: string) => `重播第 ${number} 章`,
    chapters: [
      {
        title: "你提出请求。",
        text: "打字或说出来都可以。请求会留在对话顶部，并显示它所在的项目和分支。",
        replay: "在回放中，请求位于对话顶部，消息框上方显示 orbit 文件夹和 main 分支。",
      },
      {
        title: "Sens 展示工作。",
        text: "每次工具调用都是一个步骤，按顺序排列。第一步来自 Sens：你的项目里已有的、与请求相关的代码。",
        replay: "在回放中，Worked 这一行展开成各个步骤。第一步说明 Sens 已有 greetingFor，用过 3 次；然后 Claude 读取、搜索、运行测试并编辑。焦点落在第一次测试运行上。",
      },
      {
        title: "打开任意步骤。",
        text: "命令展开后显示输出，保留程序自己的颜色。文件和行号会打开对应文件。",
        replay: "在回放中，Run npm test 这一步展开显示输出：2 个测试文件、10 个测试通过。",
      },
      {
        title: "Sens 进行检查。",
        text: "每个更改在写入磁盘前都会被检查。Claude 写了一个项目里已有的问候函数；Sens 拦下了这次写入，展示了原有的函数，Claude 便改用了它。",
        replay:
          "在回放中，Sens stopped a write 这一步展开：它复制了 src/header.tsx 第 7 行的已有代码，而函数 greetingFor 已在 src/greeting.ts 中。接着 Claude 想到：改用 greetingFor。",
      },
      {
        title: "查看更改内容。",
        text: "每次编辑都会以差异的形式进入 Changes，按文件列出，与上一次提交对比。",
        replay: "在回放中，路径 src/header.tsx 从它的 Edit 步骤移到 Changes，header.tsx 显示新增 4 行、删除 2 行，并调用了 greetingFor。",
      },
      {
        title: "阅读结果。",
        text: "回复结束后，工作会折叠成一行：Claude 做了什么，以及 Sens 拦下、复用和批准了什么。失败永远不会被折叠。",
        replay:
          "在回放中，步骤折叠成一行，结尾是 Sens: 1 stop, 1 reused, approved。回复移到前面：页眉接受一个可选的问候语并复用 greetingFor，测试全部通过，共 12 个。",
      },
    ] as Chapter[],
    engineLabel: "Sens 检查什么",
    engineTitle: "同样的工作，更少的代码。",
    engineLead: "Sens 位于 Claude Code 和你的项目之间。它在每个会话中运行，Claude 无法关闭或绕过它。",
    measured: [
      { figure: "12%", said: "相同功能下减少的代码" },
      { figure: "18%", said: "减少的 token" },
      { figure: "90 / 90", said: "个任务被接受，与不用 Sens 时相同" },
    ] as Figure[],
    engineNote: "在我们的基准测试中，使用 Claude Sonnet 5.5 在同一项目上连续完成 30 个任务后的结果。",
    methodLink: "方法与结果",
    checks: [
      {
        title: "展示你已有的代码。",
        text: "每条消息中，Sens 都会向 Claude 展示项目里与请求相关的代码，让它复用而不是重写。",
      },
      {
        title: "每个更改落盘前都会检查。",
        text: "复制已有代码（包括改名后的）、留下未使用的代码以及新增注释，都会在写入磁盘前被拦下，并说明应改为怎么做。新增依赖或删除测试会等你决定。",
      },
      {
        title: "再读一遍这一轮。",
        text: "更改通过后，审查者会查找只用一次的抽象、治标不治本的修复，以及重写项目已提供功能的地方。它的意见会发给你和 Claude，但不会拦下任何东西。",
      },
      {
        title: "最终由你决定。",
        text: "如果 Claude 三次尝试都没能让更改获得批准，这一轮就会被暂停。你可以接受、在不动 git 的情况下撤销，或让 Claude 修复。",
      },
    ] as Check[],
    languages:
      "支持三十种语言。TypeScript 和 JavaScript（含 Vue 与 Svelte）、Python、Go、Rust、Java、Kotlin、C#、PHP、Ruby、C、C++、Swift、Dart、Scala、F#、Lua、Elixir、Gleam 和 Zig 完整支持；另外 11 种语言检查复制和注释。",
    closingTitle: "给 Claude Code 一个窗口。",
    exploreOnGithub: "在 GitHub 上探索",
    windowsVersions: "Windows 10 或 11",
    claudeAccount: "一个 Claude 账户",
    unsigned: ["安装程序尚未签名，因此 Windows SmartScreen 会发出警告。请与", { link: "版本说明" }, "中的 SHA-256 进行比对。"] as Piece[],
  },
});
