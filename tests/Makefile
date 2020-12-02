
test_001_podcast.json : test_001_podcast.ttl
	rapper -i turtle $<  -o json > $<

test_001_podcast.jsonld : test_001_podcast.ttl
	python3 ../src/jsonld_cli.py $<  > $@

test_001_podcast.jsonld.flattened.url : test_001_podcast.jsonld.flattened
	python3 ../src/json_to_url.py $< > $@
