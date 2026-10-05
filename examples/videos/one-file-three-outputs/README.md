# Video 7 — one file, three outputs

`src/Talk.wf` holds the talk's content once (two constants), one component that draws a point, and three pages: `Site` (a web page), `Handout` (an A4 `Document`) and `Deck` (a `Presentation`). A project builds one kind of output, so the three come from `wf render`, which picks one page of a file and draws it as HTML, a PDF or a slide deck. `webfluent.app.json` is there so `wf check` can hold the file to every rule first. `wf render` reads its data as JSON on standard input; this file needs none, hence the `echo '{}'`.

    wf check --deny-warnings
    echo '{}' | wf render src/Talk.wf --page Site                    -o build/talk.html
    echo '{}' | wf render src/Talk.wf --page Handout --format pdf    -o build/handout.pdf
    echo '{}' | wf render src/Talk.wf --page Deck    --format slides -o build/deck.pdf
