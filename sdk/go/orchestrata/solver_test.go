package orchestrata

import "testing"

func TestSolve(t *testing.T) {
    result, err := Solve(Problem{
        Domains: [][]int64{{0, 1}},
        Constraints: []Constraint{{
            Kind: PreferValue, Left: 0, Value: 1, Weight: 1,
        }},
    }, 10)
    if err != nil {
        t.Fatal(err)
    }
    if len(result.Values) != 1 || result.Values[0] != 1 || result.Score.Soft != 0 {
        t.Fatalf("unexpected result: %+v", result)
    }
}

func TestRejectsEmptyDomain(t *testing.T) {
    _, err := Solve(Problem{Domains: [][]int64{{}}}, 10)
    if err == nil {
        t.Fatal("expected empty domain error")
    }
}
