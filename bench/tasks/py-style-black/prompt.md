`click.style("x", fg=0)` returns the text without any color: the 256-color index 0, black, is silently dropped, and the same happens with `bg=0`. Make index 0 work like any other index.
