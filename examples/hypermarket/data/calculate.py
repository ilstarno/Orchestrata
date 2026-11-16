"""Work a two-stage, independently separable hypermarket placement example."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ZONES = [
    ('ambient', 'Packaged grocery', 'ambient', 20, 420, 'dry_facing'),
    ('bulk', 'Bulk pantry', 'ambient', 40, 620, 'low_load_bay'),
    ('beverage', 'Beverages', 'ambient', 30, 460, 'bottle_facing'),
    ('chilled', 'Chilled foods', 'chilled', 16, 330, 'chiller'),
    ('frozen', 'Frozen foods', 'frozen', 16, 330, 'freezer'),
    ('household', 'Household consumables', 'ambient', 24, 480, 'separated_facing'),
]
PRODUCTS = [
    ['Pasta','Oats','Rice cakes','Cereal','Crackers','Tea','Coffee','Soup','Beans','Tomato sauce'],
    ['Rice sacks','Flour sacks','Lentils','Sugar sacks','Chickpeas','Baking mix','Cornmeal','Barley','Salt bags','Porridge oats'],
    ['Sparkling water','Still water','Apple juice','Orange juice','Iced tea','Lemon soda','Ginger soda','Cola','Tonic water','Berry drink'],
    ['Plain yogurt','Oat yogurt','Milk','Soft cheese','Cheddar','Fresh pasta','Cream','Butter','Tofu','Deli salad'],
    ['Frozen peas','Frozen berries','Frozen pizza','Frozen fries','Frozen broccoli','Frozen corn','Frozen bread','Frozen mango','Frozen spinach','Ice cream'],
    ['Laundry pods','Dish soap','Paper towels','Trash bags','Sponges','Tissues','Surface wipes','Hand soap','Dry cloths','Foil'],
]


def walk_meters(point, origin):
    return sum(abs(x-y) for x, y in zip(point, origin, strict=True))


def inputs():
    layouts = {'warehouse': {'origin': [0, 0], 'slots': []}, 'floor': {'origin': [0, 0], 'slots': []}}
    skus, demand = [], {'regular-week': {}, 'promotion-week': {}}
    for zi, (zone, category, temperature, weight_limit, height_limit, fixture) in enumerate(ZONES):
        for i, name in enumerate(PRODUCTS[zi]):
            sku = f'SKU-{len(skus)+1:03}'
            skus.append({'skuId': sku, 'label': name, 'category': category, 'zoneId': zone,
                         'storageTemperature': temperature, 'caseMassKg': weight_limit - (2+i),
                         'caseHeightMm': height_limit - (20 + 9*i), 'floorFixtureRequired': fixture})
            for stage, width, height, prefix in (('warehouse', 8, 4, 'WH'), ('floor', 6, 3, 'FL')):
                coord = [width*(1 + i//2), 2 + zi*height*2 + i%2*height]
                layouts[stage]['slots'].append({'slotId':f'{prefix}-{zone.upper()}-{i+1:02}', 'zoneId':zone, 'coordinate':coord,
                    'temperatureClass':temperature, 'maxCaseMassKg':weight_limit, 'maxCaseHeightMm':height_limit,
                    'fixtureClass': 'warehouse_bin' if stage == 'warehouse' else fixture})
            demand['regular-week'][sku] = {'pickTrips': 5 + 3*i + zi, 'replenishmentTrips': 3 + 2*i + zi%2}
            regular_rank = (i*3+zi)%10
            demand['promotion-week'][sku] = {'pickTrips': 7 + 4*regular_rank + (12 if zone in ('beverage','chilled') else 0),
                                              'replenishmentTrips': 4 + 3*((i*7+zi)%10) + (8 if zone == 'beverage' else 0)}
    return skus, layouts, demand


def eligible(sku, stage, slot):
    return {
        'departmentZoneMatches': sku['zoneId'] == slot['zoneId'],
        'temperatureStorageMatches': sku['storageTemperature'] == slot['temperatureClass'],
        'caseMassWithinLimit': sku['caseMassKg'] <= slot['maxCaseMassKg'],
        'caseHeightWithinLimit': sku['caseHeightMm'] <= slot['maxCaseHeightMm'],
        'compatibleFloorFixture': stage == 'warehouse' or sku['floorFixtureRequired'] == slot['fixtureClass'],
    }


def make_plan(skus, layout, demand, optimize):
    positions = {sku['skuId']: {'skuId': sku['skuId']} for sku in skus}
    for stage, field in [('warehouse','pickTrips'), ('floor','replenishmentTrips')]:
        for zone, *_ in ZONES:
            group = [s for s in skus if s['zoneId'] == zone]
            slots = sorted((p for p in layout[stage]['slots'] if p['zoneId'] == zone),
                           key=lambda p: (walk_meters(p['coordinate'],layout[stage]['origin']), p['slotId']))
            if optimize:
                group.sort(key=lambda sku: (-demand[sku['skuId']][field], sku['skuId']))
            assert len(group) == len(slots)
            # Group separability is a tested assumption, not an arbitrary
            # eligibility relaxation: all within-zone pairings must be legal.
            assert all(all(eligible(s, stage, p).values()) for s in group for p in slots)
            for sku, place in zip(group, slots, strict=True):
                positions[sku['skuId']][stage+'SlotId'] = place['slotId']
    return list(positions.values())


def metrics(assignments, layout, demand):
    output = {}
    for stage, trips in [('warehouse','pickTrips'), ('floor','replenishmentTrips')]:
        places = {p['slotId']:p for p in layout[stage]['slots']}
        output[stage+'Meters'] = sum(2*walk_meters(places[a[stage+'SlotId']]['coordinate'],layout[stage]['origin'])
                                     *demand[a['skuId']][trips] for a in assignments)
    return output


def run():
    skus, layout, demand = inputs()
    scenarios = []
    for name, weights in demand.items():
        baseline = make_plan(skus, layout, weights, False)
        recommended = make_plan(skus, layout, weights, True)
        basem, recm = metrics(baseline,layout,weights), metrics(recommended,layout,weights)
        scenarios.append({'id':name,'baseline':baseline,'recommended':recommended,
                          'metrics':{'baseline':basem,'recommended':recm,
                                     'avoided':{k:basem[k]-recm[k] for k in basem}}})
    return {'schemaVersion':1, 'layout':layout, 'scenarios':scenarios,
            'skus':[{k:s[k] for k in ('skuId','label','category','zoneId')} for s in skus],
            'metricsScope':'Illustrative round-trip workload-weighted walking distance, not sales or live traffic.'}, {
        'schemaVersion':1,'skus':skus,'layouts':layout,'demandProfiles':demand,
        'baselinePolicy':'Fixed SKU order maps to closest qualifying slots in each stage and area.',
        'recommendationPolicy':'Maximize trips allocated near the origin via descending trip count within each compatible placement group.',
    }


def operational(visual, fixtures):
    sku_by_id = {s['skuId']:s for s in fixtures['skus']}
    slots = {p['slotId']:p for l in visual['layout'].values() for p in l['slots']}
    runs = []
    for scenario in visual['scenarios']:
        profile = fixtures['demandProfiles'][scenario['id']]
        prior = {x['skuId']:x for x in scenario['baseline']}
        decisions = []
        for a in scenario['recommended']:
            sku = sku_by_id[a['skuId']]
            decisions.append({
                'inventoryReference': sku['skuId'],
                'productProfile': sku,
                'demandBasis': profile[sku['skuId']],
                'candidateComparison': {
                    'baselineWarehouseSlot': prior[sku['skuId']]['warehouseSlotId'],
                    'baselineShelfSlot': prior[sku['skuId']]['floorSlotId'],
                    'proposedPlacement': {
                        'receivingStorage': {'warehouseBin': slots[a['warehouseSlotId']],
                            'verification': {'hardPolicyChecks': eligible(sku,'warehouse',slots[a['warehouseSlotId']])}},
                        'shopFloor': {'shelfPosition': slots[a['floorSlotId']],
                            'verification': {'hardPolicyChecks': eligible(sku,'floor',slots[a['floorSlotId']])}},
                    },
                },
            })
        runs.append({
            'scenarioId': scenario['id'], 'handlingMetrics': scenario['metrics'],
            'decisionRegister': {'plannedSkuCount':len(decisions),'placementEntries':decisions},
        })
    return {
        'id':'hypermarket-stockflow','title':'Hypermarket warehouse slotting and shelf replenishment',
        'dataClassification':'Illustrative SKU, store grid and activity volumes; not real store performance',
        'sourceEvidence':{'planningInputs':'inputs.json','recommendedLayout':'visual.json','reproductionScript':'calculate.py',
                          'validationScript':'verify.py','liveStoreConnection':False},
        'operatingOrganization': {
            'businessPurpose':'Keep inventory replenishment within safe placement envelopes while reducing modeled travel',
            'operatingUnit': {
                'unitName':'Hypermarket inventory operations',
                'operatingFunction': {
                    'functionLabel':'Stockflow from receiving bins to shopping floor',
                    'planningWorkstream': {
                        'scopeOfOptimization':'Within six compatible stock-handling areas. Storage and shelving travel are modeled separately.',
                        'policyDefinition': {'baselineRule': fixtures['baselinePolicy'], 'recommendationRule': fixtures['recommendationPolicy'],
                                             'distanceDefinition':'Round-trip Manhattan distance from a fixed stage origin',
                                             'physicalReviewRequired':True,'metricDoesNotRepresentSales':True},
                        'planningProcedure': {
                            'availableWarehouseBins':len(visual['layout']['warehouse']['slots']),
                            'availableShelfPositions':len(visual['layout']['floor']['slots']),
                            'operatingScenarios':runs,
                        },
                    },
                },
            },
        },
    }


def dimensions(v, keys, level=0):
    if isinstance(v, dict):
        keys.update(v)
        return max([level+1]+[dimensions(x,keys,level+1) for x in v.values()])
    if isinstance(v,list):
        return max([level]+[dimensions(x,keys,level) for x in v])
    return level


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args = parser.parse_args()
    visual, fixtures = run()
    case = operational(visual, fixtures)
    props = set()
    depth = dimensions(case,props)
    assert depth >= 10 and len(props) >= 50
    for name, value in [('inputs.json',fixtures),('visual.json',visual),('operational-case.json',case)]:
        payload = json.dumps(value,ensure_ascii=False,indent=2)+'\n'
        if args.check:
            assert (ROOT/name).read_text()==payload
        else:
            (ROOT/name).write_text(payload)
    print(f'60 SKU records; 2 demand scenarios; {depth} object levels, {len(props)} distinct properties.')
    for scenario in visual['scenarios']:
        print(scenario['id'], scenario['metrics'])
