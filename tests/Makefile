
test_001_podcast.json : test_001_podcast.ttl
	rapper -i turtle test_001_podcast.ttl  -o json > $<

test_001_podcast.jsonld : test_001_podcast.ttl
	python3 ../src/jsonld_cli.py ../tests/test_001_podcast.ttl  > $@
