import click
import json
from rdflib import Graph, plugin
from rdflib.serializer import Serializer

@click.command()
@click.argument('inputf',type=click.Path(exists=True))
@click.option('--format',default='turtle')
def main(inputf,format):
    
    g = Graph().parse(inputf, format=format)
    print(g.serialize(format='json-ld', indent=4).decode("utf-8"))


if __name__ == '__main__':
    main()

