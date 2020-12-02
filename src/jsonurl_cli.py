
import click
import json
from jsonurl.jsonurl import parse_query

@click.command()
@click.argument('inputf',type=click.File('r'))
def main(inputf):
    data = json.load(inputf)
    print (jsonurl.query_string(data))

if __name__ == '__main__':
    main()

