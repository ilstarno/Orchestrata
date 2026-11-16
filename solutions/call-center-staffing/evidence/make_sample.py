"""Deterministic sample contact-center shift; no names or contact records."""

from datetime import datetime, timedelta
from zoneinfo import ZoneInfo


ZONE = ZoneInfo("America/New_York")
DAY = datetime(2026, 11, 16, tzinfo=ZONE)
SKILLS = ("billing", "recovery", "fraud_review", "product_support", "priority_triage")
LANGUAGES = ("en", "es", "fr", "pt", "de")


def timestamp(hour, minute=0):
    return (DAY + timedelta(hours=hour, minutes=minute)).isoformat(timespec="minutes")


def period(sh, sm, eh, em):
    return {"start": timestamp(sh, sm), "end": timestamp(eh, em)}


def queue(identifier, label, skill, language, demand, aht_seconds, weight):
    return {
        "id": identifier, "label": label, "channel": "voice",
        "required_agents": 1,
        "skill": {"id": skill, "minimum_rating": 4},
        "language": {"tag": language, "minimum_rating": 4},
        "minimum_role_months": 6, "soft_preference_weight": weight,
        "arrival_counts": {"typical": demand[0], "surge": demand[1]},
        "average_handle_seconds": aht_seconds,
    }


def operator(operator_id, skills, languages, months, shift, breaks=(), *, voice=1, chat=2):
    return {
        "id": operator_id, "skills": skills, "languages": languages,
        "role_experience_months": months, "shift": shift, "breaks": list(breaks),
        "contact_capacity": {"voice": voice, "chat": chat},
    }


def make_sample():
    full = period(8, 0, 16, 0)
    lunch = (period(12, 0, 12, 30),)
    handpicked = [
        ({"billing": 5}, {"en": 5, "es": 5}, 63),
        ({"billing": 4}, {"en": 4, "es": 4, "pt": 3}, 34),
        ({"billing": 4, "priority_triage": 3}, {"en": 5, "es": 2}, 42),
        ({"recovery": 5}, {"en": 4}, 26),
        ({"recovery": 4, "billing": 3}, {"en": 5, "fr": 2}, 56),
        ({"recovery": 4}, {"en": 4, "es": 4}, 12),
        ({"fraud_review": 5}, {"en": 5}, 40),
        ({"fraud_review": 4, "recovery": 3}, {"en": 4, "fr": 4}, 28),
        ({"fraud_review": 4}, {"en": 4}, 20),
        ({"product_support": 5}, {"en": 5, "fr": 5}, 48),
        ({"product_support": 4}, {"en": 3, "fr": 5}, 31),
        ({"product_support": 4}, {"en": 5, "fr": 4}, 23),
        ({"priority_triage": 5}, {"en": 5}, 39),
        ({"priority_triage": 4, "billing": 3}, {"en": 5, "es": 4}, 50),
        ({"priority_triage": 4}, {"en": 4, "fr": 4}, 15),
        ({"billing": 5}, {"en": 5, "es": 5}, 84),
        ({"recovery": 5}, {"en": 5}, 61),
        ({"billing": 5, "priority_triage": 5}, {"en": 5, "es": 5}, 72),
    ]
    operators = []
    for i, (skills, langs, experience) in enumerate(handpicked, 1):
        shift = period(10, 45, 18, 45) if i == 16 else full
        breaks = (period(10, 20, 10, 45),) if i == 17 else lunch
        operators.append(operator(f"OP-{i:03}", skills, langs, experience, shift, breaks,
                                  voice=0 if i == 18 else 1, chat=2 + (i % 2)))
    for i in range(19, 61):
        first = SKILLS[i % len(SKILLS)]
        second = SKILLS[(i + 2) % len(SKILLS)]
        extra_language = LANGUAGES[1 + (i % (len(LANGUAGES) - 1))]
        shift = (period(7, 0, 15, 0), period(12, 0, 20, 0), period(6, 0, 10, 0))[i % 3]
        breaks = (period(10, 5, 10, 25),) if i % 7 == 0 and i % 3 == 0 else ()
        operators.append(operator(
            f"OP-{i:03}", {first: 1 + i % 3, second: 1 + (i // 3) % 3},
            {"en": 2 + i % 4, extra_language: 1 + i % 5},
            3 + (i * 7) % 79, shift, breaks, voice=0 if i % 8 == 0 else 1, chat=1 + i % 3,
        ))
    return {
        "schema_version": 1, "dataset_id": "call-center-coverage-ny-sample-v1",
        "classification": "sample", "timezone": ZONE.key,
        "coverage_window": period(10, 0, 10, 30),
        "metadata": {
            "generated_by": "make_sample.py v1", "observed_data": False,
            "purpose": "One simultaneous voice queue coverage window with qualified operators",
            "rating_scale": {
                "1": "introductory", "2": "supported", "3": "routine",
                "4": "independent", "5": "advanced",
            },
            "units": {
                "probability": "basis points", "handling_time": "seconds",
                "experience": "months", "arrivals": "contacts per coverage window",
                "contact_capacity": "simultaneous contacts per operator per channel",
            },
        },
        "policy": {
            "max_queues_per_operator": 1, "evaluation_budget": 100000,
            "voice_occupancy_target_basis_points": 8000,
            "demand_scenarios_basis_points": {"typical": 7000, "surge": 3000},
        },
        "queues": [
            queue("billing-escalations", "Billing escalations", "billing", "en", (3, 5), 360, 9),
            queue("spanish-billing", "Spanish-language billing", "billing", "es", (2, 4), 420, 7),
            queue("account-recovery", "Account recovery", "recovery", "en", (3, 4), 330, 5),
            queue("fraud-review", "Fraud review", "fraud_review", "en", (2, 4), 480, 10),
            queue("french-product", "French-language product support", "product_support", "fr", (2, 3), 420, 6),
            queue("priority-support", "Priority customer support", "priority_triage", "en", (3, 5), 300, 8),
        ],
        "operators": operators,
    }
