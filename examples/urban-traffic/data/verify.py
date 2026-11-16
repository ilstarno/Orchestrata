"""Verify the stored Manhattan replay with an independent shortest-time check."""
import json
from calculate import HERE, source_graph


def load(name):
    return json.loads((HERE / name).read_text())


def bellman_ford(nodes, arcs, source, end):
    costs = {source: 0}
    for _ in range(len(nodes) - 1):
        current = costs.copy()
        for a, b, cost in arcs:
            if a in costs:
                candidate = costs[a] + cost
                if candidate < current.get(b, float('inf')):
                    current[b] = candidate
        if current == costs:
            break
        costs = current
    return costs[end]


def verify():
    fixture, extract, graph = source_graph()
    replay, geo = load('replay.json'), load('replay.geojson')
    assert replay['metadata'] == geo['metadata']
    assert replay['metadata']['sourceSha256'] == fixture['sourceSha256']
    start, goal = (p['osmNodeId'] for p in replay['endpoints'])
    features = {f['id']: f for f in geo['features']}
    assert len(features) == 7
    assert len(replay['updates']) == 2
    for pt in replay['endpoints']:
        assert pt['coordinate'] == list(extract.coords[pt['osmNodeId']])
        assert features[pt['kind']]['geometry'] == {'type': 'Point', 'coordinates': pt['coordinate']}
    for update in replay['updates']:
        nodes = update['nodeIds']
        assert nodes == features[update['id']]['properties']['nodeIds']
        assert features[update['id']]['geometry']['coordinates'] == [list(extract.coords[n]) for n in nodes]
        assert all(v in graph[u] for u,v in zip(nodes,nodes[1:]))
    baseline = replay['routes'][0]
    for route in replay['routes']:
        variant = route['variant']
        blocked, limited = set(), {}
        for update in replay['updates']:
            if variant not in update['appliesTo']:
                continue
            for a,b in zip(update['nodeIds'],update['nodeIds'][1:]):
                if update['kind'] == 'closed_segment':
                    blocked.add((a,b))
                else:
                    limited[a,b] = update['effectiveSpeedKph']
        modeled_speed = replay['metadata']['baseSpeedKph']
        weights = {(a,b): round(1000*meters / (limited.get((a,b), modeled_speed)/3.6))
                   for a,edges in graph.items() for b,meters in edges.items() if (a,b) not in blocked}
        nodes = route['nodeIds']
        assert nodes[0] == start and nodes[-1] == goal
        hops = list(zip(nodes,nodes[1:]))
        proof = bellman_ford(graph, [(a,b,c) for (a,b),c in weights.items()], start, goal)
        assert proof == sum(weights[a,b] for a,b in hops) == route['durationMs']
        assert route['durationSeconds'] == proof / 1000
        assert route['deltaSeconds'] == (proof - baseline['durationMs']) / 1000
        assert route['distanceMeters'] == round(sum(graph[a][b] for a,b in hops),1)
        assert features[variant]['geometry']['coordinates'] == [list(extract.coords[n]) for n in nodes]
        assert all(v == features[variant]['properties'][k] for k,v in route.items() if k != 'nodeIds')
        old = list(zip(baseline['nodeIds'],baseline['nodeIds'][1:]))
        assert route['baselinePathStillAvailable'] == all(e in weights for e in old)
        print(f'{variant}: {len(hops)} real directed edges; {proof} ms independent shortest-time proof')
    assert [r['durationMs'] for r in replay['routes']] == [54225,82298,147898]
    assert replay['routes'][1]['nodeIds'] == baseline['nodeIds']
    assert not replay['routes'][2]['baselinePathStillAvailable']


if __name__ == '__main__':
    verify()
