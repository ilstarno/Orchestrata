# A technician calls out. Then an urgent job arrives.

The original three-job field-service sample establishes the lift, safety and maintenance jobs and the technician eligibility. This variant adds a backup technician, explicit clock windows, sample driving durations, a callout, and an urgent job. It does not change the original fixture or claim that the original fixture had these extra fields.

One simultaneous dispatch wave only: every assigned person takes one job. Ready time, shift end, and both service window bounds are local minutes since midnight. The service start must fall inside the inclusive window. Calculate arrival from the available time and outbound drive; service can wait until the window opens. Overtime includes service and return to base. The driving estimates are fixed and symmetric by assumption. The cap is 20 minutes per person. This is a sample policy, not a recommendation or a regulatory claim.

The input to the native solver contains only the eligible technician indices, an exception option unique to each job, and typed hard/soft rules. Distinct assignments prevent double booking. The objective minimizes round-trip driving minutes + twice overtime minutes + unserved-job penalties. Higher unserved-job penalties keep urgent work from being traded away for easier driving. These are declared planning weights, not measured costs. An exception means a job was not assigned in this wave, not that it can never be done.

From this directory:

~~~sh
python3 reproduce.py --library /path/to/liborchestrata_ffi.so
~~~

This runs the native ABI v1 solver on all three normalized inputs, then compares the exact choices and scores with a separately enumerated objective on the raw alternatives. Pass --write to refresh the recorded artifact after checks pass. The checked run used the native library built from Orchestrata engine commit cddfdaee5b9ee9ff2577399726b95a12286e69c7.

Results: original 3/3 jobs, 78 driving minutes, no overtime; callout 3/3, 80 driving minutes, 20 overtime; urgent arrival after the callout 3/4, 102 driving minutes, 38 overtime across two people. The exception is preventive maintenance: its two eligible technicians are assigned to the safety inspection and urgent isolation. These are exact choices only for this small fixed input, not a speed or savings benchmark.
