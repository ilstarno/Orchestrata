"""Verify saved placements and optimum by independent subset enumeration."""
from pathlib import Path
import json
from functools import lru_cache

ROOT = Path(__file__).resolve().parent
def read(name):
    return json.loads((ROOT/name).read_text())


def minimum_assignment(trips, costs):
    @lru_cache(None)
    def best(used):
        k = used.bit_count()
        if k == len(trips):
            return 0
        return min(t*trips[k] + best(used | (1<<j)) for j,t in enumerate(costs) if not (used>>j)&1)
    return best(0)


def check():
    data, visual = read('inputs.json'), read('visual.json')
    sku = {s['skuId']:s for s in data['skus']}
    assert len(sku) == len(data['skus']) == 60
    assert data['layouts'] == visual['layout']
    assert {(s['skuId'],s['label']) for s in visual['skus']} == {(s['skuId'],s['label']) for s in sku.values()}
    for scenario in visual['scenarios']:
        d = data['demandProfiles'][scenario['id']]
        assert set(d) == set(sku)
        for stage, measure in [('warehouse','pickTrips'),('floor','replenishmentTrips')]:
            layout = data['layouts'][stage]
            slots = {x['slotId']:x for x in layout['slots']}
            assert len(slots) == len(layout['slots']) == 60
            origin_x, origin_y = layout['origin']
            cost = {k: 2*(abs(s['coordinate'][0]-origin_x)+abs(s['coordinate'][1]-origin_y)) for k,s in slots.items()}
            optimum = 0
            for zone in {s['zoneId'] for s in slots.values()}:
                trips = [d[k][measure] for k,s in sku.items() if s['zoneId']==zone]
                costs = [cost[k] for k,s in slots.items() if s['zoneId']==zone]
                assert len(trips) == len(costs) == 10
                optimum += minimum_assignment(trips,costs)
            values = {}
            for plan in ('baseline','recommended'):
                assignments = scenario[plan]
                assert len(assignments) == len(sku)
                assert {a['skuId'] for a in assignments} == set(sku)
                assert {a[stage+'SlotId'] for a in assignments} == set(slots)
                score = 0
                for a in assignments:
                    p, s = sku[a['skuId']], slots[a[stage+'SlotId']]
                    assert p['zoneId'] == s['zoneId'] and p['storageTemperature'] == s['temperatureClass']
                    assert 0 < p['caseMassKg'] <= s['maxCaseMassKg'] and 0 < p['caseHeightMm'] <= s['maxCaseHeightMm']
                    if stage=='floor':
                        assert p['floorFixtureRequired'] == s['fixtureClass']
                    score += d[p['skuId']][measure]*cost[s['slotId']]
                assert score == scenario['metrics'][plan][stage+'Meters']
                values[plan] = score
            assert values['recommended'] == optimum < values['baseline']
            assert scenario['metrics']['avoided'][stage+'Meters'] == values['baseline']-optimum
        print(scenario['id']+': all 60 SKUs checked across both layouts and both plans; placement costs independently optimal within model.')


if __name__ == '__main__':
    check()
