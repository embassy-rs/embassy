all:
	asciidoctor -d book -D thebook/ index.adoc
	cp -r images thebook

clean:
	rm -rf thebook

.PHONY: all clean
