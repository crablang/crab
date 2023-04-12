include ../tools.mk

all:
	$(CRABLANGDOC) --output-format=json x.html 2>&1 | diff - output-format-json.stderr
