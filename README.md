# osm-cli

A command line tool for interacting with the OpenStreetMap API.

## Installation

This tool is written in Rust and is published to [crates.io](https://crates.io/crates/osm-cli). If you have a Rust toolchain installed, you can download and build it by running:

```
$ cargo install osm-cli
```

You can also clone this repository and run `cargo install --path .` in it.

Note: `osm-cli` uses Rust 2024 (1.85.0). If during installation you run into an error message that says "this version of Cargo is older than the 2024 edition", you may need to update your Rust toolchain. If you installed Rust using `rustup`, run `rustup update`. Otherwise, update Rust using your system package manager.

## Features

### Fetching nodes, ways, relations, or changesets

`osm-cli` can fetch information about OSM nodes, ways, relations, or changesets.

```
$ osm node 349018659
<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6" generator="openstreetmap-cgimap 2.0.1 (3529608 spike-06.openstreetmap.org)" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
 <node id="349018659" visible="true" version="7" changeset="155467928" timestamp="2024-08-19T15:53:56Z" user="Mateusz Konieczny - bot account" uid="3199858" lat="47.4624576" lon="-121.4782337">
  <tag k="ele" v="1907.7"/>
  <tag k="gnis:feature_id" v="1521553"/>
  <tag k="name" v="Kaleetan Peak"/>
  <tag k="natural" v="peak"/>
  <tag k="source" v="USGS"/>
  <tag k="wikidata" v="Q49040648"/>
  <tag k="wikipedia" v="en:Kaleetan Peak"/>
 </node>
</osm>
```

By default, output is in OSM XML format, but you can pass `-f json` to switch to JSON output.

```
$ osm node 349018659 -f json
{
  "version": "0.6",
  "generator": "openstreetmap-cgimap 2.0.1 (2557194 spike-08.openstreetmap.org)",
  "copyright": "OpenStreetMap and contributors",
  "attribution": "http://www.openstreetmap.org/copyright",
  "license": "http://opendatacommons.org/licenses/odbl/1-0/",
  "elements": [
    {
      "type": "node",
      "id": 349018659,
      "lat": 47.4624576,
      "lon": -121.4782337,
      "timestamp": "2024-08-19T15:53:56Z",
      "version": 7,
      "changeset": 155467928,
      "user": "Mateusz Konieczny - bot account",
      "uid": 3199858,
      "tags": {
        "ele": "1907.7",
        "gnis:feature_id": "1521553",
        "name": "Kaleetan Peak",
        "natural": "peak",
        "source": "USGS",
        "wikidata": "Q49040648",
        "wikipedia": "en:Kaleetan Peak"
      }
    }
  ]
}
```

If you try to fetch an element that's been deleted, you'll get an error:

```
$ osm way 338034921
Error: https://www.openstreetmap.org/api/0.6/way/338034921: status code 410
```

But by adding the `--history` option you can get the full history of any element:

```
$ osm way 338034921 --history
<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6" generator="openstreetmap-cgimap 2.0.1 (2270284 spike-07.openstreetmap.org)" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
 <way id="338034921" visible="true" version="1" changeset="30148985" timestamp="2015-04-11T18:37:52Z" user="bdiscoe" uid="402624">
  <nd ref="546919059"/>
  <nd ref="3450963445"/>
  <nd ref="3450963444"/>
  <nd ref="3450963443"/>
  <nd ref="3450963442"/>
  <nd ref="3450963441"/>
  <nd ref="3450963440"/>
  <nd ref="3450963439"/>
  <nd ref="3450963438"/>
  <nd ref="3450963437"/>
  <nd ref="3450963436"/>
  <nd ref="3450963435"/>
  <nd ref="263565911"/>
  <tag k="name" v="Housatonic River"/>
  <tag k="waterway" v="river"/>
 </way>
 <way id="338034921" visible="true" version="2" changeset="148425172" timestamp="2024-03-09T14:24:53Z" user="Mashin" uid="187467">
  <nd ref="546919059"/>
  <nd ref="3450963445"/>
  <nd ref="3450963444"/>
  <nd ref="3450963443"/>
  <nd ref="3450963442"/>
  <nd ref="3450963441"/>
  <nd ref="3450963440"/>
  <nd ref="11708472051"/>
  <nd ref="3450963439"/>
  <nd ref="3450963438"/>
  <nd ref="3450963437"/>
  <nd ref="3450963436"/>
  <nd ref="3450963435"/>
  <nd ref="263565911"/>
  <tag k="name" v="Housatonic River"/>
  <tag k="waterway" v="river"/>
 </way>
 <way id="338034921" visible="false" version="3" changeset="148484987" timestamp="2024-03-11T01:01:11Z" user="quincylvania" uid="4515353"/>
</osm>
```

Fetching a changeset prints the changeset metadata by default (the timestamp, author, comment, etc).

```
$ osm changeset 155530622
<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6" generator="openstreetmap-cgimap 2.0.1 (2557176 spike-08.openstreetmap.org)" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
 <changeset id="155530622" created_at="2024-08-20T21:36:16Z" closed_at="2024-08-20T21:36:17Z" open="false" user="jake-low" uid="8794039" min_lat="47.6647943" min_lon="-121.2881568" max_lat="47.6647943" max_lon="-121.2881568" comments_count="0" changes_count="1">
  <tag k="changesets_count" v="1992"/>
  <tag k="comment" v="Skykomish, WA: add name, operator, and website to Necklace Valley Trailhead"/>
  <tag k="created_by" v="iD 2.29.0"/>
  <tag k="host" v="https://www.openstreetmap.org/edit"/>
  <tag k="imagery_used" v="Bing Maps Aerial"/>
  <tag k="locale" v="en-US"/>
 </changeset>
</osm>
```

By adding `--diff`, you get the osmChange XML instead (containing the new versions of elements that were modified).

```$λ osm changeset 155530622 --diff
<?xml version="1.0" encoding="UTF-8"?>
<osmChange version="0.6" generator="openstreetmap-cgimap 2.0.1 (2557185 spike-08.openstreetmap.org)" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
 <modify>
  <node id="2523603738" visible="true" version="3" changeset="155530622" timestamp="2024-08-20T21:36:16Z" user="jake-low" uid="8794039" lat="47.6647943" lon="-121.2881568">
   <tag k="highway" v="trailhead"/>
   <tag k="name" v="Necklace Valley Trailhead"/>
   <tag k="operator" v="US Forest Service"/>
   <tag k="website" v="https://www.fs.usda.gov/recarea/mbs/recarea/?recid=80228"/>
  </node>
 </modify>
</osmChange>
```

### Subscribing to changes from a replication server

The `osm replication` subcommand is useful if you want to subscribe to OSM change files from a replication server. The command itself handles figuring out what replication files are available, and prints out their sequence numbers, timestamps, and URLs. It's up to you to download the files you need and process them.

Here's an example which uses the `osm replication` command to print all of the minutely replication files available `--since` a given timestamp. (You can also use `--seqno` to print files since a given sequence number).

```
$ osm replication minute --since 2025-03-23T00:00:00Z
6524866 2025-03-23T00:00:00Z https://planet.openstreetmap.org/replication/minute/006/524/866.osc.gz
6524867 2025-03-23T00:01:00Z https://planet.openstreetmap.org/replication/minute/006/524/867.osc.gz
6524868 2025-03-23T00:02:01Z https://planet.openstreetmap.org/replication/minute/006/524/868.osc.gz
... lots more output omitted ...
```

You can pipe this output into a script that downloads the replication file and does something useful with it, like applying the changes to a local database. By default, `osm replication` prints all of the replication files that are currently available and then exits. If you want the command to continue running forever, printing a new output line each time a new file becomes available, use the `--watch` option.

```
$ osm replication minute --since 2024-11-23T00:00:00Z --watch \
  | xargs -d '\n' -n 1 ./update-osmx.sh
```

update-osmx.sh:

```
#!/bin/sh

seqno=$1
timestamp=$2
url=$3

# download and decompress the replication file (containing a batch of edits to OSM)
curl --fail -s -S -L $url | gzip -d > /tmp/changes.osc
# apply the changes to a local OSMExpress database
# (https://github.com/bdon/OSMExpress)
osmx update planet.osmx /tmp/changes.osc $seqno $timestamp --commit
```

In addition to minutely replication diffs (osmChange XML files, containing OSM edits), `osm replication` can also listen for changeset replication files, which contain metadata about changesets that are uploaded to OSM.

```
$ osm replication changesets --since 2025-03-23T00:00:00Z
6442440 2025-03-23T00:00:16Z https://planet.openstreetmap.org/replication/changesets/006/442/441.osm.gz
6442441 2025-03-23T00:01:17Z https://planet.openstreetmap.org/replication/changesets/006/442/442.osm.gz
6442442 2025-03-23T00:02:17Z https://planet.openstreetmap.org/replication/changesets/006/442/443.osm.gz
... lots more output omitted ...
```

## License

Code for this tool is available under the terms of the ISC License. See the LICENSE file for details.
