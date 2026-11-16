"""Replay one illustrative vehicle detour against the local Manhattan OSM snapshot."""
from hashlib import file_digest
from pathlib import Path
import sys
import json

HERE = Path(__file__).resolve().parent
OSM = HERE.parents[3] / 'orch-multicity-streets'
sys.path.insert(0, str(OSM))
from route_xml import StreetExtract, graph_for, shortest_path  # noqa: E402

CASE_ID = 'NYC-TRAFFIC-01'
ORIGIN, DESTINATION = 9051115967, 8265964372
SLOW_NODES = [42428232, 9032581896, 6223969247, 11355997383, 11766442644, 11766442643, 7369969942, 11766442642, 9198484656, 42428227]
CLOSED = (42428227, 9198484657)
DEFAULT_SPEED_KPH = 30
REDUCED_SPEED_KPH = 8
VARIANTS = ('baseline', 'slowdown', 'closure')


def source_graph():
    f = json.loads((OSM / 'fixtures/new-york.json').read_text())
    p = OSM / 'sources/new-york.osm'
    with p.open('rb') as stream:
        if file_digest(stream, 'sha256').hexdigest() != f['sourceSha256']:
            raise ValueError('Expected the pinned New York OSM snapshot')
    extract = StreetExtract(p, f['bbox'])
    return f, extract, graph_for('vehicle', extract)


def speed_variant(graph, variant):
    closed = {CLOSED} if variant == 'closure' else set()
    slow = set(zip(SLOW_NODES, SLOW_NODES[1:])) if variant != 'baseline' else set()
    return {a: {b: travel_ms(m, REDUCED_SPEED_KPH if (a, b) in slow else DEFAULT_SPEED_KPH)
            for b, m in edges.items() if (a, b) not in closed} for a, edges in graph.items()}


def travel_ms(meters, speed):
    return round(meters * 3600 / speed)


def feature(i, kind, coordinates, props):
    return {'type': 'Feature', 'id': i, 'geometry': {'type': kind, 'coordinates': coordinates}, 'properties': props}


def calculate():
    source, extract, graph = source_graph()
    coords = extract.coords
    assert all(v in graph[u] for u,v in zip(SLOW_NODES,SLOW_NODES[1:]))
    assert CLOSED[1] in graph[CLOSED[0]]
    routes, features = [], []
    for variant in VARIANTS:
        costs = speed_variant(graph, variant)
        path, ms = shortest_path(costs, ORIGIN, DESTINATION)
        hops = list(zip(path,path[1:]))
        meters = sum(graph[u][v] for u,v in hops)
        baseline_hops = list(zip(routes[0]['nodeIds'],routes[0]['nodeIds'][1:])) if routes else hops
        baseline_available = all(v in costs[u] for u,v in baseline_hops)
        baseline_ms = sum(costs[u][v] for u,v in baseline_hops) if baseline_available else None
        props = {
            'variant': variant, 'distanceMeters': round(meters,1), 'durationSeconds': ms / 1000,
            'durationMs': int(ms), 'deltaSeconds': (ms - routes[0]['durationMs']) / 1000 if routes else 0,
            'distanceDeltaMeters': round(meters - routes[0]['distanceMeters'], 1) if routes else 0,
            'baselinePathStillAvailable': baseline_available,
            'baselinePathDurationMsAtUpdatedSpeeds': baseline_ms,
            'decision': 'keep_route' if hops == baseline_hops else 'use_detour',
        }
        routes.append({**props, 'nodeIds': path})
        features.append(feature(variant,'LineString',[coords[n] for n in path],{'kind':'route',**props}))
    updates = [
        {'id':'UPD-01', 'kind':'reduced_speed', 'nodeIds': SLOW_NODES, 'appliesTo':['slowdown','closure'],
         'effectiveSpeedKph': REDUCED_SPEED_KPH, 'direction':'as_listed'},
        {'id':'UPD-02', 'kind':'closed_segment', 'nodeIds': list(CLOSED), 'appliesTo':['closure'],
         'effectiveSpeedKph': None, 'direction':'as_listed'},
    ]
    for update in updates:
        features.append(feature(update['id'],'LineString',[coords[n] for n in update['nodeIds']],update))
    endpoints = []
    for kind, n in [('origin',ORIGIN),('destination',DESTINATION)]:
        endpoints.append({'kind': kind, 'osmNodeId': n, 'coordinate': coords[n]})
        features.append(feature(kind,'Point',coords[n],{'kind':kind,'osmNodeId':n}))
    metadata = {
        'caseId': CASE_ID, 'city': 'New York · Midtown Manhattan', 'bbox': source['bbox'],
        'source': source['source'], 'sourceSha256': source['sourceSha256'],
        'attribution': '© OpenStreetMap contributors', 'license': 'ODbL 1.0',
        'licenseUrl': 'https://www.openstreetmap.org/copyright', 'fixMapUrl':'https://www.openstreetmap.org/fixthemap',
        'updateSource': 'Illustrative update inputs; not a claim about real or historical traffic events.',
        'mode': 'vehicle', 'baseSpeedKph': DEFAULT_SPEED_KPH,
        'coverage': 'One origin and destination; results are optimal only for the retained directed graph and modeled costs.',
        'routingLimits': 'No live traffic, lane capacity, signal delay, legality review, or certified navigation.',
    }
    return {'schemaVersion':1,'metadata':metadata,'endpoints':endpoints,'updates':updates,'routes':routes}, {
      'type':'FeatureCollection','metadata':metadata,'features':features
    }


def main():
    replay, geo = calculate()
    for name, data in [('replay.json',replay),('replay.geojson',geo)]:
        (HERE/name).write_text(json.dumps(data, ensure_ascii=False, indent=2)+'\n')


if __name__ == '__main__':
    main()
