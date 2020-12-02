import click
import json
from nested_query_string import NestedQueryString


@click.command()
@click.argument('inputf',type=click.File('r'))
def main(inputf):
    data = json.load(inputf)
    print (NestedQueryString.encode(data))

if __name__ == '__main__':
    main()

