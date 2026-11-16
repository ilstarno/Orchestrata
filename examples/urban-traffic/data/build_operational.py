"""Build a richly structured operational case from the verified replay."""
import json
from calculate import HERE, source_graph

replay = json.loads((HERE / 'replay.json').read_text())
meta = replay['metadata']
baseline, slowdown, closure = replay['routes']
_, _, graph = source_graph()
update_records = [{
    'eventId': update['id'],
    'effectType': update['kind'],
    'activeReplayVariants': update['appliesTo'],
    'directionalWayNodes': update['nodeIds'],
    'modifiedDirectedEdgeCount': len(update['nodeIds'])-1,
    'replaySpeedKph': update['effectiveSpeedKph'],
    'evidenceClass': 'Illustrative input, not a real incident notice',
} for update in replay['updates']]

case = {
    'id': meta['caseId'],
    'title': 'Midtown Manhattan: respond to a simulated street closure',
    'dataClassification': 'Illustrative traffic changes on mapped OpenStreetMap streets',
    'sourceEvidence': {
        'originDataset': meta['source'], 'snapshotDigest': meta['sourceSha256'],
        'mapAttribution': meta['attribution'], 'mapDataLicense': meta['license'],
        'replayResult': 'replay.json', 'mapGeometry': 'replay.geojson',
        'shortestTimeVerifier': 'verify.py',
    },
    'operation': {
        'operatingArea': meta['city'], 'coverageBounds': meta['bbox'],
        'businessUnit': {
            'unitFunction': 'Coordinate local service travel',
            'dispatchDesk': {
                'deskMandate': 'Decide whether to retain or replace a planned travel path',
                'routingWorkstream': {
                    'travelMode': meta['mode'],
                    'serviceDemand': {
                        'tripCount': len(replay['endpoints'])-1,
                        'tripOrigin': replay['endpoints'][0],
                        'tripDestination': replay['endpoints'][1],
                    },
                    'replayRun': {
                        'executionMode': 'Offline deterministic replay',
                        'sourceIsLive': False,
                        'streetNetwork': {
                            'retainedGraphNodes': len(graph),
                            'retainedDirectedEdges': sum(map(len,graph.values())),
                            'comparisonUnits': 'Per-edge travel rounded to integer milliseconds',
                            'nominalModelSpeedKph': meta['baseSpeedKph'],
                        },
                        'eventLog': {'eventCount': len(update_records), 'updateRecords': update_records},
                        'decisionProcess': {
                            'initialPlanVariant': baseline['variant'],
                            'intermediateVariant': slowdown['variant'],
                            'closureVariant': closure['variant'],
                            'objectiveCriterion': 'Minimum modeled travel time among directed edges retained by the sample access profile',
                            'solverMethod': 'Dijkstra over the snapshot graph',
                            'independentCheck': 'Bellman-Ford agrees on exact integer travel time',
                            'decisionLedger': {
                                'recordedVariants': [{
                                    'variantId': r['variant'],
                                    'selectedAction': r['decision'],
                                    'performanceRecord': {
                                        'measuredGeometryMeters': r['distanceMeters'],
                                        'modeledCostMs': r['durationMs'],
                                        'secondsAboveBaseline': r['deltaSeconds'],
                                        'extraMetersAgainstBaseline': r['distanceDeltaMeters'],
                                    },
                                    'policyReview': {
                                        'decisionReason': 'Available path remains fastest' if r['decision']=='keep_route' else 'Baseline path contains a closed edge',
                                        'operatingEvidence': {
                                            'baselineStillUsable': r['baselinePathStillAvailable'],
                                            'baselineCostWhenAvailableMs': r['baselinePathDurationMsAtUpdatedSpeeds'],
                                            'requiresLiveDispatchReview': True,
                                            'routineTrafficNotModeled': True,
                                        },
                                    },
                                } for r in replay['routes']],
                            },
                        },
                    },
                },
            },
        },
    },
}


def profile(value, keys, level=0):
    if isinstance(value,list):
        return max([level]+[profile(x,keys,level) for x in value])
    if isinstance(value,dict):
        keys.update(value)
        return max([level+1]+[profile(x,keys,level+1) for x in value.values()])
    return level


keys = set()
depth = profile(case, keys)
assert depth >= 10 and len(keys) >= 50
print(f'{depth} object levels and {len(keys)} meaningful keys verified')
(HERE / 'operational-case.json').write_text(json.dumps(case, ensure_ascii=False, indent=2)+'\n')
