## Welcome to the escaped RDFa namespace

[docs/1.0.html](docs/1.0.html)

## Tests

### Test 1 plain old rdfa in html

* [Standard RDFA](tests/test_001_podcast.html)
* [Standard RDFA turtle](tests/test_001_podcast.ttl)

* [Blogger HTML Test](https://stream-random.blogspot.com/2020/11/test001podcasthtml.html) Blogger passes this through but it fails RDFa test.
* [Testing with RDFa validator fails](https://www.w3.org/2012/pyRdfa/extract?uri=https%3A%2F%2Fstream-random.blogspot.com%2F2020%2F11%2Ftest001podcasthtml.html&format=turtle&rdfagraph=output&vocab_expansion=false&rdfa_lite=false&embedded_rdf=true&space_preserve=true&vocab_cache=true&vocab_cache_report=false&vocab_cache_refresh=false)
* [Wordpress RDFa test](https://streamofrandompodcast.wordpress.com/2020/11/26/test-of-plain-old-rdfa/) Wordpress Kills all the RDFa, this is useless.

### Test 2 escaped the RDFa

Using the [html escaper](https://www.freeformatter.com/html-escape.html) we create an escaped RDfa in html for injecting in the blog.

* [Escaped podcast example](tests/test_002_podcast_escaped.html)
* [Escaped blogger rss](tests/test_002_podcast_escaped_blogger.rss)
* [Escaped wordpress rss](tests/test_002_podcast_escaped_wordpress.rss)

### Test 3 example rdfa
Here we just define a simple [example rdfa](tests/test_003_example.html)
This is what the rdfa extractor [returns](tests/test_003_example.ttl) but when this is implemented we will actually not return any graph data for the example.

### Test 4 escaped RDFa
Using the [html escaper](https://www.freeformatter.com/html-escape.html) we create an escaped RDfa in html for injecting in the blog.

Here we just define a simple [example rdfa](tests/test_004_example_escaped.html)
And we have the [turtle](tests/test_004_example_escaped.ttl).

## Embedded RSS.

In exploring the possiblities of simple free hosting of PHP I encountered multiple issues getting the right libraries installed and would like to fall back on 
core php 7 for the implemention. For this purpose we want to skip the dependancy of https://www.easyrdf.org/ for parsing the RDFa documents and embed RSS directly.


### Test 6 of embedded RSS

This is for the entry of a podcast.

[html](tests/test_006_erss_embed.html).
[rss](tests/test_006_erss_embed.rss)

### Test 6 of embedded RSS channel header 

This is for the header of the podcast 

[html](tests/test_006_erss_embed.html).
[rss](tests/test_006_erss_embed.rss)


### Test of encoding in query parameters

Using the https://json-ld.org/playground/ we convert the [lsonld](tests/test_001_podcast.jsonld) into a compacter form.
Then we use a patched version of https://github.com/Escaped-RDFa/nested-query-string to encode the data into query strings and the utility
(json_to_url)[src/json_to_url.py] so that a query string encoded data in be passed to a program in a readable url. 
I just tacked on the server name as an example. 

looks like this [example](https://escaped-rdfa.github.io/processor?@graph[][@id]=file%3A///opt/podcast/namespace/tests/test_001_podcast.ttl&@graph[][https://escaped-rdfa.github.io/namespace/docs/1.0.html#embedded][@id]=_%3Aub1bL9C19&@graph[][@id]=_%3Aub1bL9C19&@graph[][@type]=http%3A//purl.org/rss/1.0/rss&@graph[][http://purl.org/rss/1.0/channel][@id]=_%3Aub1bL10C25&@graph[][@id]=_%3Aub1bL10C25&@graph[][@type]=http%3A//purl.org/rss/1.0/item&@graph[][http://purl.org/dc/terms/creator]=Jim%20Dupont&@graph[][http://purl.org/rss/1.0/enclosure]=&@graph[][http://purl.org/rss/1.0/guid]=1b593f7b-67e7-454c-bb80-32e8e32ada06&@graph[][http://purl.org/rss/1.0/link]=https%3A//anchor.fm/stre/episodes/StreamOfRandom-S3-The-Refinement-EP1-Quality-efo717&@graph[][http://purl.org/rss/1.0/pubDate]=Mon%2C%2022%20Jun%202020%2001%3A02%3A15%20GMT&@graph[][http://purl.org/rss/1.0/title]=StreamOfRandom%20S3%20The%20Refinement%20EP1%20Quality&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdduration]=986&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdepisode]=1&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdepisodeType]=full&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdexplicit]=No&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdimage]=&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdseason]=3&@graph[][http://www.itunes.com/dtds/podcast-1.0#dtdsummary]=Introduction%20to%20season%203)
