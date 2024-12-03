all:
	asciidoctor -d book -D book/ index.adoc
	cp -r images book

clean:
	rm -rf book

.PHONY: all clean
