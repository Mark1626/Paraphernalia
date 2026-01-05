--------------------------- MODULE await_on_list ---------------------------

EXTENDS Integers, Sequences, TLC

Writers == 5

(*

--algorithm await_on_list {

variables
    results        = [writer \in 1..Writers |-> FALSE];
    allow_writing  = FALSE;
    write_detected = FALSE;
    total          = 0;

process (reader = 0)
variable j = 0;
{
    R0:         allow_writing := TRUE;

    R1:         await \E writer \in 1..Writers : (results[writer] = TRUE);

    R2:         j := 1;
    R3:         while (j <= Writers) {
                    if (results[j] = TRUE) {
                        results[j] := FALSE;
                        total := total + 1;
                    };
                    j := j + 1;
                };
                if (total # Writers) goto R2; 

    R4:         assert total = Writers;
                assert \A writer \in 1..Writers : (results[writer] = FALSE); 
                write_detected := TRUE;
} \* end process

process (writer \in 1..Writers) {
    W0:         await allow_writing;
    
    W1:         results[self] := TRUE;
} \* end process

} \* end algorithm

*)
\* BEGIN TRANSLATION (chksum(pcal) = "f73befc3" /\ chksum(tla) = "6df5df4e")
VARIABLES results, allow_writing, write_detected, total, pc, j

vars == << results, allow_writing, write_detected, total, pc, j >>

ProcSet == {0} \cup (1..Writers)

Init == (* Global variables *)
        /\ results = [writer \in 1..Writers |-> FALSE]
        /\ allow_writing = FALSE
        /\ write_detected = FALSE
        /\ total = 0
        (* Process reader *)
        /\ j = 0
        /\ pc = [self \in ProcSet |-> CASE self = 0 -> "R0"
                                        [] self \in 1..Writers -> "W0"]

R0 == /\ pc[0] = "R0"
      /\ allow_writing' = TRUE
      /\ pc' = [pc EXCEPT ![0] = "R1"]
      /\ UNCHANGED << results, write_detected, total, j >>

R1 == /\ pc[0] = "R1"
      /\ \E writer \in 1..Writers : (results[writer] = TRUE)
      /\ pc' = [pc EXCEPT ![0] = "R2"]
      /\ UNCHANGED << results, allow_writing, write_detected, total, j >>

R2 == /\ pc[0] = "R2"
      /\ j' = 1
      /\ pc' = [pc EXCEPT ![0] = "R3"]
      /\ UNCHANGED << results, allow_writing, write_detected, total >>

R3 == /\ pc[0] = "R3"
      /\ IF j <= Writers
            THEN /\ IF results[j] = TRUE
                       THEN /\ results' = [results EXCEPT ![j] = FALSE]
                            /\ total' = total + 1
                       ELSE /\ TRUE
                            /\ UNCHANGED << results, total >>
                 /\ j' = j + 1
                 /\ pc' = [pc EXCEPT ![0] = "R3"]
            ELSE /\ IF total # Writers
                       THEN /\ pc' = [pc EXCEPT ![0] = "R2"]
                       ELSE /\ pc' = [pc EXCEPT ![0] = "R4"]
                 /\ UNCHANGED << results, total, j >>
      /\ UNCHANGED << allow_writing, write_detected >>

R4 == /\ pc[0] = "R4"
      /\ Assert(total = Writers, 
                "Failure of assertion at line 34, column 17.")
      /\ Assert(\A writer \in 1..Writers : (results[writer] = FALSE), 
                "Failure of assertion at line 35, column 17.")
      /\ write_detected' = TRUE
      /\ pc' = [pc EXCEPT ![0] = "Done"]
      /\ UNCHANGED << results, allow_writing, total, j >>

reader == R0 \/ R1 \/ R2 \/ R3 \/ R4

W0(self) == /\ pc[self] = "W0"
            /\ allow_writing
            /\ pc' = [pc EXCEPT ![self] = "W1"]
            /\ UNCHANGED << results, allow_writing, write_detected, total, j >>

W1(self) == /\ pc[self] = "W1"
            /\ results' = [results EXCEPT ![self] = TRUE]
            /\ pc' = [pc EXCEPT ![self] = "Done"]
            /\ UNCHANGED << allow_writing, write_detected, total, j >>

writer(self) == W0(self) \/ W1(self)

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == reader
           \/ (\E self \in 1..Writers: writer(self))
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 

=============================================================================
\* Modification History
\* Last modified Fri Jan 02 16:10:29 IST 2026 by nimalanm
\* Created Fri Jan 02 15:44:30 IST 2026 by nimalanm
