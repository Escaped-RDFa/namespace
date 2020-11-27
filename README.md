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
