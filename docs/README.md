# embassy docs

The documentation hosted at [https://embassy.dev/book](https://embassy.dev/book). Building the documentation requires the [asciidoctor](https://asciidoctor.org/) tool, and can be built like this:

```
asciidoctor -d book -D thebook/ index.adoc
```

Then open the generated file `thebook/index.html`.
